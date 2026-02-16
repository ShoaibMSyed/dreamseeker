use std::f32::consts::PI;

use avian3d::{
    character_controller::move_and_slide::DepenetrationConfig,
    prelude::{Collider, LinearVelocity, MoveAndSlide, ShapeCastConfig, SpatialQueryFilter},
};
use bevy::{
    color::palettes::tailwind,
    post_process::{
        bloom::Bloom,
        dof::{DepthOfField, DepthOfFieldMode},
        effect_stack::ChromaticAberration,
    },
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use dreamseeker_util::observers;

use crate::{
    GameState,
    collision::GameLayer,
    input::camera::{CenterCamera, MoveCamera, Pause, Tp},
    ui::screen::{ScreenCommandsExt, pause::PauseScreen, teleport::TeleportScreen},
    util::angle::{Angle, AsAngle},
};

use super::{PLAYER_HEIGHT, Player, controller::PlayerController};

const ZOOM_SPEED: f32 = 1.5;
const MIN_DISTANCE: f32 = 4.0;
const MAX_DISTANCE: f32 = 8.0;
const PAN_SPEED: f32 = 90.0;
const CENTER_SPEED: f32 = 8.0;

const MIN_FOV: f32 = 70f32.to_radians();
const MAX_FOV: f32 = 100f32.to_radians();
const FOV_SPEED: f32 = 8.0;

const PLAYER_SPEED_SLOW: f32 = 5.5;
const PLAYER_SPEED_FAST: f32 = 9.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            PlayerCamera::system,
            PlayerCamera::follow_player,
            PlayerCamera::set_fov,
        )
            .run_if(not(in_state(GameState::Paused))),
    );
}

#[derive(Component, Reflect)]
#[require(Camera3d, SpatialListener)]
pub struct PlayerCamera {
    pub zoom: f32,

    pub distance: f32,
    pub rotation: Angle,

    pub visual_rotation: Angle,
    pub visual_speed: f32,

    pub follow_speed: f32,

    #[reflect(ignore)]
    pub collider: Collider,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            zoom: 0.0,

            distance: 5.0,
            rotation: default(),

            visual_rotation: default(),
            visual_speed: 1.0,

            follow_speed: 8.0,

            collider: Collider::sphere(0.25),
        }
    }
}

impl PlayerCamera {
    pub fn bundle() -> impl Bundle {
        (
            Self::default(),
            crate::input::camera::actions(),
            Camera {
                clear_color: ClearColorConfig::Custom(tailwind::PINK_100.into()),
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                fov: MIN_FOV,
                ..default()
            }),
            DistanceFog {
                color: Color::linear_rgb(219.0 / 255.0, 151.0 / 255.0, 209.0 / 255.0),
                falloff: FogFalloff::Linear {
                    start: 20.0,
                    end: 50.0,
                },
                ..default()
            },
            Bloom::NATURAL,
            DepthOfField {
                aperture_f_stops: 0.15,
                mode: DepthOfFieldMode::Gaussian,
                focal_distance: 6.0,
                ..default()
            },
            ChromaticAberration {
                intensity: 0.05,
                ..default()
            },
            observers![Self::on_center, Self::on_move, Self::on_pause, Self::on_tp],
        )
    }

    fn offset(&self) -> Vec3 {
        let vert = Quat::from_axis_angle(Vec3::X, 26f32.to_radians());
        let rot = Quat::from_axis_angle(Vec3::Y, self.visual_rotation.get()) * vert;
        rot * (Vec3::NEG_Z * self.distance)
    }

    fn apply(&mut self, cstick: Vec2, dt: f32) {
        if self.visual_rotation != self.rotation {
            return;
        }

        let zoom_step = ZOOM_SPEED * -cstick.y * dt;
        let pan_step = PAN_SPEED.to_radians() * cstick.x * dt;

        self.zoom = (self.zoom + zoom_step).clamp(0.0, 1.0);

        let distance_interval = MAX_DISTANCE - MIN_DISTANCE;
        self.distance = MIN_DISTANCE + distance_interval * self.zoom;

        self.rotation += pan_step.as_angle();
        self.visual_rotation = self.rotation;
    }

    fn center(&mut self, facing: Angle) {
        self.rotation = facing;
        self.visual_speed = self.rotation.diff(self.visual_rotation) * CENTER_SPEED;
    }

    fn update(&mut self, dt: f32) {
        if self.visual_rotation == self.rotation {
            return;
        }

        let step = self.visual_speed * dt;

        let old = self.visual_rotation.get();
        let new = old + step;

        // Hack to fix bug where camera is forever centering
        if old == new {
            self.visual_rotation = self.rotation;
            return;
        }

        let min = f32::min(old, new);
        let max = f32::max(old, new);

        let range = min..=max;

        let r1 = self.rotation.get();
        let r2 = self.rotation.get() + 2.0 * PI;
        let r3 = self.rotation.get() - 2.0 * PI;

        if range.contains(&r1) || range.contains(&r2) || range.contains(&r3) {
            self.visual_rotation = self.rotation;
        } else {
            self.visual_rotation = new.as_angle();
        }
    }

    fn on_center(
        _: On<Fire<CenterCamera>>,
        mut camera: Single<&mut PlayerCamera>,
        pc: Single<&PlayerController>,
    ) {
        camera.center(pc.facing);
    }

    fn on_move(
        event: On<Fire<MoveCamera>>,
        mut camera: Single<&mut PlayerCamera>,
        time: Res<Time>,
    ) {
        camera.apply(event.value, time.delta_secs());
    }

    fn on_pause(_: On<Start<Pause>>, mut cmd: Commands, state: Res<State<GameState>>) {
        if state.get() == &GameState::Paused {
            cmd.pop_screen();
        } else if state.get() == &GameState::InGame {
            cmd.push_screen(PauseScreen::bundle());
        }
    }

    fn on_tp(_: On<Start<Tp>>, mut cmd: Commands, state: Res<State<GameState>>) {
        if state.get() == &GameState::InGame {
            cmd.push_screen(TeleportScreen::bundle());
        }
    }

    fn system(mut camera: Single<&mut PlayerCamera>, time: Res<Time>) {
        camera.update(time.delta_secs());
    }

    fn follow_player(
        mut player_pos: Local<Vec3>,
        mut camera: Single<(&mut Transform, &PlayerCamera), Without<Player>>,
        player: Single<(&Transform, &Collider, Entity), With<Player>>,
        time: Res<Time>,
        mas: MoveAndSlide,
    ) {
        let target = player.0.translation
            + Vec3::Y * (-player.1.shape().as_cuboid().unwrap().half_extents.y + PLAYER_HEIGHT);

        let pp = *player_pos;
        *player_pos += (target - pp) * (1.0 - f32::exp(-camera.1.follow_speed * time.delta_secs()));

        let camera_offset = camera.1.offset();

        // let out = mas.move_and_slide(
        //     &camera.1.collider,
        //     *player_pos,
        //     Quat::default(),
        //     camera_offset,
        //     Duration::from_secs(1),
        //     &MoveAndSlideConfig::default(),
        //     &SpatialQueryFilter::from_excluded_entities([player.2])
        //         .with_mask(GameLayer::Level),
        //     |_| MoveAndSlideHitResponse::Accept,
        // );

        // camera.0.translation = out.position;

        let offset = mas.depenetrate(
            &camera.1.collider,
            *player_pos,
            Quat::default(),
            &DepenetrationConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([player.2]).with_mask(GameLayer::Level),
        );

        let hit = mas.spatial_query.cast_shape(
            &camera.1.collider,
            *player_pos + offset,
            Quat::default(),
            Dir3::new(camera_offset).unwrap_or(Dir3::Z),
            &ShapeCastConfig::from_max_distance(camera_offset.length()),
            &SpatialQueryFilter::from_excluded_entities([player.2]).with_mask(GameLayer::Level),
        );

        match hit {
            None => camera.0.translation = *player_pos + camera_offset + offset,
            Some(hit) => {
                camera.0.translation =
                    *player_pos + camera_offset.normalize_or_zero() * hit.distance + offset;
            }
        }

        camera.0.look_at(*player_pos, Dir3::Y);
    }

    fn set_fov(
        mut fov: Local<f32>,
        player: Single<&LinearVelocity, (With<PlayerController>, Changed<LinearVelocity>)>,
        mut camera: Single<&mut Projection, With<PlayerCamera>>,
        time: Res<Time>,
    ) {
        let Projection::Perspective(proj) = &mut **camera else {
            return;
        };

        let interval = PLAYER_SPEED_FAST - PLAYER_SPEED_SLOW;
        let percent = ((player.xz().length() - PLAYER_SPEED_SLOW) / interval).clamp(0.0, 1.0);

        let fov_interval = MAX_FOV - MIN_FOV;
        let target_fov = MIN_FOV + fov_interval * percent;

        *fov += (target_fov - *fov) * (1.0 - f32::exp(-FOV_SPEED * time.delta_secs()));

        proj.fov = *fov;
    }
}
