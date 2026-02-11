use std::f32::consts::PI;

use avian3d::prelude::{Collider, LinearVelocity};
use bevy::{anti_alias::fxaa::Fxaa, core_pipeline::prepass::DepthPrepass, prelude::*};
use bevy_enhanced_input::prelude::*;
use dreamseeker_util::observers;

use crate::{
    input::camera::{CenterCamera, MoveCamera},
    util::angle::{Angle, AsAngle},
};

use super::{PLAYER_HEIGHT, Player, controller::PlayerController};

const ZOOM_SPEED: f32 = 1.5;
const MIN_DISTANCE: f32 = 4.0;
const MAX_DISTANCE: f32 = 8.0;
const PAN_SPEED: f32 = 90.0;
const CENTER_SPEED: f32 = 8.0;

const FOLLOW_SPEED: f32 = 8.0;

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
        ),
    );
}

#[derive(Component, Reflect)]
#[require(Camera3d, DepthPrepass, Msaa::Off, Fxaa)]
pub struct PlayerCamera {
    pub zoom: f32,

    pub distance: f32,
    pub rotation: Angle,

    pub visual_rotation: Angle,
    pub visual_speed: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            zoom: 0.0,

            distance: 5.0,
            rotation: default(),

            visual_rotation: default(),
            visual_speed: 1.0,
        }
    }
}

impl PlayerCamera {
    pub fn bundle() -> impl Bundle {
        (
            Self::default(),
            crate::input::camera::actions(),
            Projection::Perspective(PerspectiveProjection {
                fov: MIN_FOV,
                ..default()
            }),
            observers![Self::on_center, Self::on_move,],
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

    fn system(mut camera: Single<&mut PlayerCamera>, time: Res<Time>) {
        camera.update(time.delta_secs());
    }

    fn follow_player(
        mut player_pos: Local<Vec3>,
        mut camera: Single<(&mut Transform, &PlayerCamera), Without<Player>>,
        player: Single<(&Transform, &Collider), With<Player>>,
        time: Res<Time>,
    ) {
        let target = player.0.translation
            + Vec3::Y * (-player.1.shape().as_cuboid().unwrap().half_extents.y + PLAYER_HEIGHT);

        let pp = *player_pos;
        *player_pos += (target - pp) * (1.0 - f32::exp(-FOLLOW_SPEED * time.delta_secs()));
        camera.0.translation = *player_pos + camera.1.offset();

        camera.0.look_at(target, Dir3::Y);
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
