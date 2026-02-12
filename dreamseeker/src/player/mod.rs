use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, CollisionStart, DebugRender, LinearVelocity, ShapeCastConfig, SpatialQuery,
    SpatialQueryFilter,
};
use bevy::{
    audio::{PlaybackMode, Volume},
    pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt},
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_enhanced_input::prelude::Start;
use dreamseeker_util::{construct::Make, observers};

use crate::input::player::Attack;

use self::{
    controller::{
        JumpState, PlayerController, PlayerControllerMessage, PlayerControllerSettings, PlayerState,
    },
    sword::Sword,
};

pub mod camera;
mod controller;
mod sword;

const PLAYER_HEIGHT: f32 = 1.7;
const PLAYER_WIDTH: f32 = 0.35;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((self::camera::plugin, self::controller::plugin))
        .init_resource::<PlayerSounds>()
        .add_systems(
            Update,
            (
                Player::rotate_model,
                Player::animate,
                Player::update_shadow,
                Player::play_sounds,
                Player::update_attack_state,
            ),
        );
}

#[derive(Resource)]
struct PlayerSounds {
    jump: Handle<AudioSource>,
    coyote_time_jump: Handle<AudioSource>,
    coyote_friction_jump: Handle<AudioSource>,
    air_jump: Handle<AudioSource>,
}

impl FromWorld for PlayerSounds {
    fn from_world(world: &mut World) -> Self {
        Self {
            jump: world.load_asset("jump.ogg"),
            coyote_time_jump: world.load_asset("coyote_time_jump.ogg"),
            coyote_friction_jump: world.load_asset("coyote_friction_jump.ogg"),
            air_jump: world.load_asset("air_jump.ogg"),
        }
    }
}

#[derive(Component, Reflect, Default)]
struct PlayerShadow;

#[derive(Component, Reflect, Default)]
struct PlayerModel {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    run: AnimationNodeIndex,
    slide_start: AnimationNodeIndex,
    slide: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    jump: AnimationNodeIndex,
    fall: AnimationNodeIndex,
    spin: AnimationNodeIndex,
    slash: AnimationNodeIndex,
    slam: AnimationNodeIndex,
    aplayer: Option<Entity>,
}

#[derive(Reflect, Default, PartialEq, Eq)]
enum AttackState {
    #[default]
    None,
    Spin,
    Normal,
}

#[derive(Component, Reflect, Default)]
#[require(Name::new("Player"), PlayerController, InheritedVisibility)]
pub struct Player {
    attack_state: AttackState,
}

impl Player {
    pub fn bundle() -> impl Bundle {
        (
            Self::default(),
            crate::input::player::actions(),
            // AddMesh(Cuboid::new(0.5, 1.5, 0.5)),
            // AddMaterial(Color::linear_rgb(0.1, 0.3, 0.8)),
            Collider::cuboid(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH),
            observers![Self::on_attack],
            children![
                (
                    Make(Self::make_model),
                    Transform::from_xyz(0.0, -PLAYER_HEIGHT / 2.0, 0.0),
                    observers![Self::setup_animations],
                ),
                (
                    ForwardDecal,
                    PlayerShadow,
                    Transform::from_scale(Vec3::splat(PLAYER_WIDTH)),
                    Make(Self::make_shadow),
                ),
            ],
        )
    }

    fn on_attack(_: On<Start<Attack>>, mut player: Single<(&mut Player, &PlayerState)>) -> Result {
        if player.0.attack_state == AttackState::None && matches!(player.1, PlayerState::Air(_)) {
            player.0.attack_state = AttackState::Spin;
        }
        if player.0.attack_state == AttackState::None
            && matches!(player.1, PlayerState::Grounded(_))
        {
            player.0.attack_state = AttackState::Normal;
        }

        Ok(())
    }

    fn make_model(
        assets: Res<AssetServer>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
    ) -> Result<impl Bundle + use<>> {
        let mut model = PlayerModel::default();

        let mut graph = AnimationGraph::new();

        let mut load = |i| {
            graph.add_clip(
                assets.load(format!("player.glb#Animation{i}")),
                1.0,
                graph.root,
            )
        };
        model.idle = load(1);
        model.run = load(3);
        model.slide_start = load(7);
        model.slide = load(6);
        model.walk = load(9);
        model.fall = load(0);
        model.jump = load(2);
        model.spin = load(8);
        model.slash = load(5);
        model.slam = load(4);

        model.graph = graphs.add(graph);

        Ok((SceneRoot(assets.load("player.glb#Scene0")), model))
    }

    fn make_shadow(
        mut decals: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
        assets: Res<AssetServer>,
    ) -> Result<impl Bundle + use<>> {
        Ok(MeshMaterial3d(decals.add(ForwardDecalMaterial {
            base: StandardMaterial {
                // base_color: Color::linear_rgb(0.1, 0.3, 0.8),
                base_color_texture: Some(assets.load("shadow.png")),
                ..default()
            },
            extension: ForwardDecalMaterialExt {
                depth_fade_factor: 1.0,
            },
        })))
    }

    fn setup_animations(
        event: On<SceneInstanceReady>,
        mut model: Query<&mut PlayerModel>,
        children: Query<&Children>,
        names: Query<&Name>,
        mut aplayer: Query<&mut AnimationPlayer>,
        mut cmd: Commands,
    ) -> Result {
        let mut model = model.get_mut(event.entity)?;
        for child in children.iter_descendants(event.entity) {
            if let Ok(name) = names.get(child)
                && name.as_str() == "Hand.R"
            {
                cmd.spawn((
                    Sword::bundle(),
                    ChildOf(child),
                    Transform::from_xyz(0.1, 0.2, 0.0)
                        .with_rotation(Quat::from_axis_angle(Vec3::Z, -PI / 2.0)),
                    DebugRender::default(),
                    observers![Self::on_sword_collision],
                ));
            }

            let Ok(mut aplayer) = aplayer.get_mut(child) else {
                continue;
            };

            model.aplayer = Some(child);

            aplayer.play(model.idle).repeat();

            cmd.entity(child)
                .insert(AnimationGraphHandle(model.graph.clone()));
        }

        Ok(())
    }

    fn rotate_model(
        player: Single<&PlayerController, Changed<PlayerController>>,
        mut model: Single<&mut Transform, With<PlayerModel>>,
    ) {
        let angle = player.facing;
        model.rotation = Quat::from_axis_angle(Vec3::Y, angle.get() - PI / 2.0);
    }

    fn animate(
        mut player: Single<(
            &PlayerController,
            &PlayerState,
            &LinearVelocity,
            &PlayerControllerSettings,
            &mut Player,
        )>,
        model: Single<&PlayerModel>,
        mut aplayer: Query<&mut AnimationPlayer>,
    ) -> Result {
        let Some(aplayer_entity) = model.aplayer else {
            return Ok(());
        };

        let mut aplayer = aplayer.get_mut(aplayer_entity)?;
        if matches!(player.4.attack_state, AttackState::Normal) {
            if let Some(anim) = aplayer.animation(model.slash)
                && anim.is_finished()
            {
                player.4.attack_state = AttackState::None;
                aplayer.stop(model.slash);
            } else if !aplayer.is_playing_animation(model.slash) {
                // aplayer.stop_all();
                aplayer.play(model.slash);
            }
        } else if matches!(player.4.attack_state, AttackState::Spin) {
            if let Some(anim) = aplayer.animation(model.spin)
                && anim.is_finished()
            {
                player.4.attack_state = AttackState::None;
            } else if !aplayer.is_playing_animation(model.spin) {
                aplayer.stop_all();
                aplayer.play(model.spin);
            }
        } else if matches!(player.1, PlayerState::Air(_)) {
            if player.2.y > 0.0 {
                if !aplayer.is_playing_animation(model.jump) {
                    aplayer.stop_all();
                    aplayer.play(model.jump).repeat();
                }
            } else {
                if !aplayer.is_playing_animation(model.fall) {
                    aplayer.stop_all();
                    aplayer.play(model.fall).repeat();
                }
            }
        } else if matches!(player.1, PlayerState::Slam(_)) {
            if !aplayer.is_playing_animation(model.slam) {
                aplayer.stop_all();
                aplayer.play(model.slam);
            }
        } else if player.2.xz().length_squared() == 0.0 {
            if !aplayer.is_playing_animation(model.idle) {
                aplayer.stop_all();
                aplayer.play(model.idle).repeat();
            }
        } else if player.2.xz().length() < player.3.run_speed - 1.0
            && matches!(player.1, PlayerState::Grounded(_))
        {
            if !aplayer.is_playing_animation(model.walk) {
                aplayer.stop_all();
                aplayer.play(model.walk).repeat();
            }
        } else if matches!(player.1, PlayerState::Sliding { .. }) {
            if !aplayer.is_playing_animation(model.slide_start) {
                if !aplayer.is_playing_animation(model.slide) {
                    aplayer.stop_all();
                    aplayer.play(model.slide_start);
                } else if let Some(anim) = aplayer.animation(model.slide_start)
                    && anim.is_finished()
                {
                    aplayer.play(model.slide).repeat();
                }
            }
        } else {
            if !aplayer.is_playing_animation(model.run) {
                aplayer.stop_all();
                aplayer.play(model.run).repeat();
            }
        }

        if let Some(anim) = aplayer.animation(model.spin) {
            if anim.is_finished() {
                aplayer.stop(model.spin);
            }
        }

        Ok(())
    }

    fn update_shadow(
        player: Single<(Entity, &PlayerController, &Transform, &Collider), Changed<Transform>>,
        mut shadow: Single<
            (&mut Transform, &mut Visibility),
            (With<PlayerShadow>, Without<PlayerController>),
        >,
        spatial: SpatialQuery,
    ) {
        let hit = spatial.cast_shape(
            &player.3,
            player.2.translation,
            Quat::default(),
            Dir3::NEG_Y,
            &ShapeCastConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([player.0]),
        );

        match hit {
            None => {
                *shadow.1 = Visibility::Hidden;
            }
            Some(hit) => {
                *shadow.1 = Visibility::Visible;

                shadow.0.translation.y =
                    -hit.distance - player.3.shape().as_cuboid().unwrap().half_extents.y;
            }
        }
    }

    fn play_sounds(
        player: Single<&Transform, With<Player>>,
        mut msg: MessageReader<PlayerControllerMessage>,
        sounds: Res<PlayerSounds>,
        mut cmd: Commands,
    ) {
        for msg in msg.read() {
            let sound = match msg {
                PlayerControllerMessage::GroundJump => sounds.jump.clone(),
                PlayerControllerMessage::CoyoteTimeJump => sounds.coyote_time_jump.clone(),
                PlayerControllerMessage::CoyoteFrictionJump => sounds.coyote_friction_jump.clone(),
                PlayerControllerMessage::AirJump => sounds.air_jump.clone(),
                _ => continue,
            };

            cmd.spawn((
                AudioPlayer::new(sound),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    spatial: true,
                    volume: Volume::Linear(8.0),
                    ..default()
                },
                player.clone(),
            ));
        }
    }

    fn update_attack_state(mut player: Single<(&mut Player, &PlayerState)>) {
        if player.0.attack_state == AttackState::Spin && player.1.grounded() {
            player.0.attack_state = AttackState::None;
        }
        if player.0.attack_state == AttackState::Normal && !player.1.grounded() {
            player.0.attack_state = AttackState::None;
        }
    }

    fn on_sword_collision(
        _: On<CollisionStart>,
        mut player: Single<(
            &mut Player,
            &mut LinearVelocity,
            &PlayerControllerSettings,
            &mut PlayerState,
        )>,
    ) {
        if player.0.attack_state == AttackState::Spin
            && player.1.y < player.2.min_sword_bounce
            && let PlayerState::Air(state) = &mut *player.3
        {
            state.air_jumps = 0;
            state.dashed = false;
            state.jump_state = JumpState::None;

            player.1.y = (-player.1.y).max(player.2.min_sword_bounce);
        }
    }
}
