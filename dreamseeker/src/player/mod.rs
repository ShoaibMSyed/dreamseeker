use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, LinearVelocity, ShapeCastConfig, SpatialQuery, SpatialQueryFilter,
};
use bevy::{
    pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt},
    prelude::*,
    scene::SceneInstanceReady,
};
use dreamseeker_util::{construct::Make, observers};

use self::controller::{PlayerController, PlayerState};

pub mod camera;
mod controller;

const PLAYER_HEIGHT: f32 = 1.7;
const PLAYER_WIDTH: f32 = 0.35;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((self::camera::plugin, self::controller::plugin))
        .add_systems(
            Update,
            (Player::rotate_model, Player::animate, Player::update_shadow),
        );
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
    aplayer: Option<Entity>,
}

#[derive(Component, Reflect, Default)]
#[require(PlayerController, InheritedVisibility)]
pub struct Player;

impl Player {
    pub fn bundle() -> impl Bundle {
        (
            Self,
            crate::input::player::actions(),
            // AddMesh(Cuboid::new(0.5, 1.5, 0.5)),
            // AddMaterial(Color::linear_rgb(0.1, 0.3, 0.8)),
            Collider::cuboid(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH),
            children![
                (
                    Make(Self::make_model),
                    Transform::from_xyz(0.0, -PLAYER_HEIGHT / 2.0, 0.0),
                    observers![Self::on_scene_ready]
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

    fn make_model(
        assets: Res<AssetServer>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
    ) -> Result<impl Bundle + use<>> {
        let mut model = PlayerModel::default();

        let mut graph = AnimationGraph::new();
        model.idle = graph.add_clip(assets.load("player.glb#Animation0"), 1.0, graph.root);
        model.run = graph.add_clip(assets.load("player.glb#Animation1"), 1.0, graph.root);
        model.slide_start = graph.add_clip(assets.load("player.glb#Animation3"), 1.0, graph.root);
        model.slide = graph.add_clip(assets.load("player.glb#Animation2"), 1.0, graph.root);

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

    fn on_scene_ready(
        event: On<SceneInstanceReady>,
        mut model: Query<&mut PlayerModel>,
        children: Query<&Children>,
        mut aplayer: Query<&mut AnimationPlayer>,
        mut cmd: Commands,
    ) -> Result {
        let mut model = model.get_mut(event.entity)?;
        for child in children.iter_descendants(event.entity) {
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
        player: Single<(&PlayerController, &PlayerState, &LinearVelocity)>,
        model: Single<&PlayerModel>,
        mut aplayer: Query<&mut AnimationPlayer>,
    ) -> Result {
        let Some(aplayer_entity) = model.aplayer else {
            return Ok(());
        };

        let mut aplayer = aplayer.get_mut(aplayer_entity)?;
        if player.2.xz().length_squared() == 0.0 {
            if !aplayer.is_playing_animation(model.idle) {
                aplayer.stop_all();
                aplayer.play(model.idle).repeat();
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
}
