use std::{f32::consts::PI, time::Duration};

use avian3d::{character_controller::move_and_slide::DepenetrationConfig, prelude::*};
use bevy::{ecs::query::QueryData, prelude::*};
use bevy_enhanced_input::prelude::*;

use crate::{
    GameState,
    collision::GameLayer,
    input::player::{Dash, Jump, Move, Slide, Walk, WallGrab},
    player::{PLAYER_HEIGHT, PLAYER_WIDTH},
    util::angle::Angle,
};

use super::{PlayerModel, camera::PlayerCamera};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<PlayerControllerMessage>()
        .add_systems(Update, PlayerInput::flycam)
        .add_systems(
            FixedUpdate,
            (
                PlayerInput::gather,
                PlayerController::step,
                PlayerController::set_collider,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
}

#[derive(Component, Reflect)]
pub struct PlayerControllerSettings {
    pub gravity: f32,
    pub floor_snap: f32,
    pub step: f32,
    pub jump: f32,
    pub min_floor_angle: f32,
    pub maximum_grounded_up_velocity: f32,

    pub flycam: bool,

    pub coyote_time: f32,
    pub air_friction: f32,
    pub terminal_velocity: f32,

    pub air_speed: f32,
    pub air_accel: f32,

    pub air_jumps: u8,
    pub air_jump_forward_boost: f32,

    pub dash_enabled: bool,
    pub dash_velocity: f32,
    pub dash_height: f32,

    /// When faster than `run_speed`, have no friction when landing on the ground for `coyote_friction` frames
    pub coyote_friction: u8,
    pub run_speed: f32,

    pub slide_enabled: bool,
    pub slide_speed: f32,
    pub slide_time: f32,

    pub slam_enabled: bool,
    pub slam_pause: f32,
    pub slam_velocity: f32,
    pub slam_jump_boost: f32,

    pub wall_grab_enabled: bool,
    pub wall_grab_min_normal: f32,
    pub wall_grab_max_normal: f32,
    pub wall_grab_max_wall_distance: f32,
    pub wall_grab_max_away_velocity: f32,

    pub wall_jump_add_vertical: f32,
    pub wall_jump_max_vertical: f32,
    pub wall_jump_add_horizontal: f32,

    pub min_sword_bounce: f32,
}

impl Default for PlayerControllerSettings {
    fn default() -> Self {
        Self {
            gravity: 20.0,
            floor_snap: 0.25,
            step: 0.33,
            jump: 1.0,
            min_floor_angle: 0.7,
            maximum_grounded_up_velocity: 5.8,

            flycam: false,

            coyote_time: (1.0 / 64.0) * 5.0,
            air_friction: 0.3,
            terminal_velocity: 20.0,

            air_speed: 5.5,
            air_accel: 8.0,

            air_jumps: 2,
            air_jump_forward_boost: 4.5,

            dash_enabled: true,
            dash_velocity: 10.0,
            dash_height: 0.5,

            coyote_friction: 3,
            run_speed: 5.5,

            slide_enabled: true,
            slide_speed: 7.5,
            slide_time: 0.7,

            slam_enabled: true,
            slam_pause: 0.5,
            slam_velocity: 20.0,
            slam_jump_boost: 2.0,

            wall_grab_enabled: true,
            wall_grab_min_normal: -0.1,
            wall_grab_max_normal: 0.6,
            wall_grab_max_wall_distance: 0.2,
            wall_grab_max_away_velocity: 2.0,

            wall_jump_add_vertical: 1.0,
            wall_jump_max_vertical: 5.0,
            wall_jump_add_horizontal: 5.0,

            min_sword_bounce: 10.0,
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct PlayerInput {
    pub movement: Vec2,
    pub speed_modifier: f32,
    pub jump: ActionEvents,
    pub slide: ActionEvents,
    pub dash: ActionEvents,
    pub wall_grab: ActionEvents,
}

impl PlayerInput {
    fn flycam(mut pcs: Single<&mut PlayerControllerSettings>, keys: Res<ButtonInput<KeyCode>>) {
        if keys.just_pressed(KeyCode::KeyH) {
            pcs.flycam = !pcs.flycam;
        }
    }

    fn gather(
        mut pi: Single<(&mut PlayerController, &mut PlayerInput, &PlayerState)>,
        camera: Single<&PlayerCamera>,
        walk: Single<&ActionState, With<Action<Walk>>>,
        jump: Single<&ActionEvents, With<Action<Jump>>>,
        slide: Single<&ActionEvents, With<Action<Slide>>>,
        dash: Single<&ActionEvents, With<Action<Dash>>>,
        wall_grab: Single<&ActionEvents, With<Action<WallGrab>>>,
        movement: Single<&Action<Move>>,
    ) {
        let dir = Vec3::new(movement.x, 0.0, -movement.y)
            .normalize_or_zero()
            .rotate_y((camera.rotation + Angle::new(std::f32::consts::PI)).get());
        pi.1.movement.x = dir.x;
        pi.1.movement.y = dir.z;
        pi.1.speed_modifier = movement.length();
        if pi.1.speed_modifier < 0.3 {
            pi.1.speed_modifier = 0.0;
        } else if pi.1.speed_modifier < 0.7 || **walk == ActionState::Fired {
            pi.1.speed_modifier = 0.5;
        } else {
            pi.1.speed_modifier = 1.0;
        }

        if pi.1.movement.length_squared() > 0.0 && !pi.2.facing_locked() {
            let angle = Vec2::new(pi.1.movement.y, pi.1.movement.x).to_angle();
            pi.0.facing = Angle::new(angle);
        }

        pi.1.jump = **jump;
        pi.1.slide = **slide;
        pi.1.dash = **dash;
        pi.1.wall_grab = **wall_grab;
    }
}

#[derive(Reflect, Clone, Default, PartialEq, Eq)]
pub enum JumpState {
    #[default]
    None,
    Normal,
    Halved,
}

#[derive(Reflect, Clone, Default)]
pub struct GroundedState {
    pub frame_count: u8,
    pub jump_boost: bool,
}

#[derive(Reflect, Clone, Default)]
pub struct AirState {
    pub air_jumps: u8,
    pub dashed: bool,
    pub jump_state: JumpState,
    pub coyote_countdown: f32,
}

impl AirState {
    fn with_coyote(mut self, settings: &PlayerControllerSettings) -> Self {
        self.coyote_countdown = settings.coyote_time;
        self
    }
}

#[derive(Reflect, Clone, Default)]
pub struct SlidingState {
    direction: Vec2,
    timer: f32,
}

#[derive(Reflect, Clone, Default)]
pub struct SlamState {
    timer: f32,
}

#[derive(Reflect, Clone)]
pub struct WallGrabState {
    wall_normal: Dir3,
    prev_air_state: AirState,
}

impl WallGrabState {
    fn new(wall_normal: Dir3, prev_air_state: AirState) -> Self {
        Self {
            wall_normal,
            prev_air_state,
        }
    }
}

#[derive(Component, Reflect)]
pub enum PlayerState {
    Grounded(GroundedState),
    Air(AirState),
    Sliding(SlidingState),
    Slam(SlamState),
    WallGrab(WallGrabState),
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Air(AirState::default())
    }
}

impl PlayerState {
    pub fn grounded(&self) -> bool {
        matches!(self, Self::Grounded(_) | Self::Sliding(_))
    }

    pub fn facing_locked(&self) -> bool {
        matches!(self, Self::Sliding(_) | Self::Slam(_) | Self::WallGrab(_))
    }
}

#[derive(Component, Reflect, Default)]
#[require(
    Collider,
    RigidBody::Kinematic,
    CollisionLayers::new(GameLayer::Player, LayerMask::ALL),
    TransformInterpolation,
    CustomPositionIntegration,
    SleepingDisabled,
    PlayerInput,
    PlayerState,
    PlayerControllerSettings
)]
pub struct PlayerController {
    pub facing: Angle,
}

impl PlayerController {
    fn step(
        query: Query<(MovementData, &mut PlayerState)>,
        mut mas: MoveAndSlide,
        mut msg: MessageWriter<PlayerControllerMessage>,
        time: Res<Time>,
    ) {
        for (move_data, mut state) in query {
            let mut mover = Mover::new(move_data, &mut mas, &mut msg, time.delta());

            mover.step(&mut *state);
        }
    }

    fn set_collider(
        mut sliding: Local<bool>,
        mut player: Single<
            (
                &PlayerState,
                &mut Collider,
                &mut Transform,
                &mut PlayerController,
            ),
            Changed<PlayerState>,
        >,
        mut model: Single<&mut Transform, (With<PlayerModel>, Without<PlayerState>)>,
    ) {
        let last_sliding = *sliding;
        *sliding = matches!(player.0, PlayerState::Sliding { .. });

        if last_sliding && !*sliding {
            *player.1 = Collider::cuboid(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH);
            player.2.translation.y += PLAYER_HEIGHT / 4.0;
            model.translation.y -= PLAYER_HEIGHT / 4.0;
        } else if !last_sliding && *sliding {
            *player.1 = Collider::cuboid(PLAYER_WIDTH, PLAYER_HEIGHT / 2.0, PLAYER_WIDTH);
            player.2.translation.y -= PLAYER_HEIGHT / 4.0;
            model.translation.y += PLAYER_HEIGHT / 4.0;
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MovementData {
    entity: Entity,
    transform: &'static mut Transform,
    velocity: &'static mut LinearVelocity,
    pc: &'static mut PlayerController,
    input: &'static PlayerInput,
    settings: &'static PlayerControllerSettings,
    collider: &'static Collider,
}

#[derive(Deref, DerefMut)]
struct Mover<'a, 'w1, 's1, 'w2, 's2, 'w3> {
    #[deref]
    data: MovementDataItem<'w1, 's1>,
    mas: &'a mut MoveAndSlide<'w2, 's2>,
    msg: &'a mut MessageWriter<'w3, PlayerControllerMessage>,
    delta: Duration,
    dt: f32,
    filter: SpatialQueryFilter,
}

impl<'a, 'w, 's, 'w2, 's2, 'w3> Mover<'a, 'w, 's, 'w2, 's2, 'w3> {
    fn new(
        data: MovementDataItem<'w, 's>,
        mas: &'a mut MoveAndSlide<'w2, 's2>,
        msg: &'a mut MessageWriter<'w3, PlayerControllerMessage>,
        delta: Duration,
    ) -> Self {
        Self {
            filter: SpatialQueryFilter::from_excluded_entities([data.entity])
                .with_mask(GameLayer::Level),
            data,
            mas,
            msg,
            dt: delta.as_secs_f32(),
            delta,
        }
    }

    fn step(&mut self, state: &mut PlayerState) {
        if self.settings.flycam {
            let mut dir = vec3(self.input.movement.x, 0.0, self.input.movement.y);
            if self.input.jump.contains(ActionEvents::FIRE) {
                dir.y += 1.0;
            }
            if self.input.slide.contains(ActionEvents::FIRE) {
                dir.y -= 1.0;
            }
            self.data.transform.translation += dir * self.dt * 20.0;

            return;
        }

        self.half_gravity(state);

        self.update_state(state);

        self.snap_to_floor(state);
        self.half_gravity(state);
        self.check_grounded(state);
    }

    fn update_state(&mut self, state: &mut PlayerState) {
        match state {
            PlayerState::Grounded(_) => self.update_grounded(state),
            PlayerState::Air(_) => self.update_air(state),
            PlayerState::Sliding(_) => self.update_sliding(state),
            PlayerState::Slam(_) => self.update_slam(state),
            PlayerState::WallGrab(_) => self.update_wall_grab(state),
        }
    }

    fn update_grounded(&mut self, state: &mut PlayerState) {
        let PlayerState::Grounded(gstate) = state else {
            return;
        };

        let coyote_friction = gstate.frame_count < self.data.settings.coyote_friction;

        if coyote_friction {
            gstate.frame_count += 1;
        }

        let too_fast = self.velocity.xz().length() > self.settings.run_speed + 0.5;

        if too_fast {
            if !coyote_friction {
                // TODO: Tickrate independent friction
                self.velocity.0 /= 1.5;
            }

            let attempted_velocity = self.velocity.0
                + vec3(self.input.movement.x, 0.0, self.input.movement.y)
                    * self.input.speed_modifier
                    * self.settings.run_speed;

            let cur_speed = self.velocity.length();

            if let Ok(dir) = Dir3::new(attempted_velocity) {
                self.velocity.0 = dir * cur_speed;
            }
        } else {
            self.velocity.x =
                self.input.movement.x * self.input.speed_modifier * self.settings.run_speed;
            self.velocity.z =
                self.input.movement.y * self.input.speed_modifier * self.settings.run_speed;
        }

        self.ground_move();

        self.velocity.y = 0.0;

        if self.input.jump.contains(ActionEvents::START) {
            self.ground_jump(state);

            if too_fast && coyote_friction {
                self.msg.write(PlayerControllerMessage::CoyoteFrictionJump);
            } else {
                self.msg.write(PlayerControllerMessage::GroundJump);
            }
        } else if self.input.slide.contains(ActionEvents::START) && self.settings.slide_enabled {
            let dir = Vec2::from_angle(self.pc.facing.get());
            *state = PlayerState::Sliding(SlidingState {
                direction: Vec2::new(dir.y, dir.x),
                timer: 0.0,
            });
        }
    }

    fn update_air(&mut self, state: &mut PlayerState) {
        let PlayerState::Air(astate) = state else {
            return;
        };

        self.air_friction();

        astate.coyote_countdown = (astate.coyote_countdown - self.dt).max(0.0);

        if astate.jump_state == JumpState::Normal
            && self.data.velocity.y > 0.0
            && !self.data.input.jump.contains(ActionEvents::FIRE)
        {
            self.data.velocity.y /= 2.0;
            astate.jump_state = JumpState::Halved;
        }

        if astate.jump_state != JumpState::None && self.data.velocity.y <= 0.0 {
            astate.jump_state = JumpState::None;
        }

        // Wall Grab
        if self.data.settings.wall_grab_enabled
            && self.data.input.wall_grab.contains(ActionEvents::FIRE)
        {
            if let Some(wall_normal) = self.try_wall_grab() {
                let prev_state = astate.clone();
                *state = PlayerState::WallGrab(WallGrabState::new(wall_normal, prev_state));
                return;
            }
        }

        // Slam
        if self.data.settings.slam_enabled && self.data.input.slide.contains(ActionEvents::START) {
            *state = PlayerState::Slam(default());
            return;
        }

        // Dash
        if self.data.settings.dash_enabled
            && !astate.dashed
            && self.data.input.dash.contains(ActionEvents::START)
        {
            astate.dashed = true;

            let dir = Vec2::from_angle(self.data.pc.facing.get());
            let dir = Dir3::new(vec3(dir.y, 0.0, dir.x)).unwrap_or(Dir3::Z);

            let hvel = vec3(self.data.velocity.x, 0.0, self.data.velocity.z);
            let speed_towards_dir = speed_towards_dir(hvel, dir);

            let boost = (self.data.settings.dash_velocity - speed_towards_dir).max(0.0);
            self.data.velocity.0 += dir * boost;
            self.data.velocity.y =
                (2.0 * self.data.settings.gravity * self.data.settings.dash_height).sqrt();
        }

        // Jumping
        if self.data.input.jump.contains(ActionEvents::START) {
            if astate.air_jumps < self.data.settings.air_jumps && astate.coyote_countdown <= 0.0 {
                astate.air_jumps += 1;

                // Air Jump
                self.data.velocity.y =
                    (2.0 * self.data.settings.gravity * self.data.settings.jump).sqrt();
                astate.air_jumps += 1;
                astate.jump_state = JumpState::Normal;

                // Forward Boost

                if let Ok(dir) = Dir3::new(vec3(
                    self.data.input.movement.x,
                    0.0,
                    self.data.input.movement.y,
                )) {
                    let hvel = vec3(self.data.velocity.x, 0.0, self.data.velocity.z);
                    let speed_towards_dir = speed_towards_dir(hvel, dir);

                    let boost = (self.settings.air_jump_forward_boost - speed_towards_dir).max(0.0);
                    self.data.velocity.0 += dir * boost;
                }

                self.msg.write(PlayerControllerMessage::AirJump);
            } else if astate.coyote_countdown > 0.0 {
                // Coyote Time
                self.ground_jump(state);

                self.msg.write(PlayerControllerMessage::CoyoteTimeJump);
            }
        }

        self.air_move();

        self.apply_velocity(false, |_| {});
    }

    fn update_sliding(&mut self, state: &mut PlayerState) {
        let PlayerState::Sliding(sstate) = state else {
            return;
        };

        sstate.timer += self.dt;

        self.data.velocity.x = sstate.direction.x * self.data.settings.slide_speed;
        self.data.velocity.z = sstate.direction.y * self.data.settings.slide_speed;

        if sstate.timer >= self.data.settings.slide_time {
            *state = PlayerState::Grounded(default());
        }

        if self.input.jump.contains(ActionEvents::START) {
            self.velocity.x *= 1.5;
            self.velocity.z *= 1.5;
            self.ground_jump(state);
        }

        self.ground_move();
    }

    fn update_slam(&mut self, state: &mut PlayerState) {
        let PlayerState::Slam(sstate) = state else {
            return;
        };

        **self.data.velocity = Vec3::ZERO;

        if sstate.timer < self.data.settings.slam_pause {
            sstate.timer += self.dt;
        }

        if sstate.timer >= self.data.settings.slam_pause {
            self.velocity.y = -self.data.settings.slam_velocity;
        }

        let min_floor_angle = self.settings.min_floor_angle;

        let mut hit_point = None;

        self.apply_velocity(false, |hit| {
            if hit.normal.y >= min_floor_angle {
                hit_point = Some(hit.point);
            }
        });

        if let Some(hit_point) = hit_point {
            self.msg.write(PlayerControllerMessage::Slam(hit_point));
            self.check_grounded(state);
            if let PlayerState::Grounded(gstate) = state {
                gstate.jump_boost = true;
            }
        }
    }

    fn update_wall_grab(&mut self, state: &mut PlayerState) {
        let PlayerState::WallGrab(wstate) = state else {
            return;
        };

        wstate.prev_air_state.air_jumps = 0;

        self.velocity.0 = Vec3::ZERO;

        // Update facing
        self.pc.facing = Angle::new(vec2(-wstate.wall_normal.z, -wstate.wall_normal.x).to_angle());

        // Stop wall grabbing without input
        if !self.input.wall_grab.contains(ActionEvents::FIRE) {
            let air_state = wstate.prev_air_state.clone();

            *state = PlayerState::Air(AirState {
                air_jumps: air_state.air_jumps,
                dashed: air_state.dashed,
                jump_state: JumpState::None,
                coyote_countdown: 0.0,
            });

            self.apply_velocity(false, |_| {});

            return;
        }

        // Wall Jump
        if self.input.jump.contains(ActionEvents::START) {
            let air_state = wstate.prev_air_state.clone();

            if self.velocity.y < self.settings.wall_jump_max_vertical {
                let max_add = self.settings.wall_jump_max_vertical - self.velocity.y;
                let vadd =
                    (self.settings.gravity * 2.0 * self.settings.wall_jump_add_vertical).sqrt();
                let vadd = vadd.min(max_add);

                self.velocity.y += vadd;
            }

            let hadd = wstate.wall_normal * self.settings.wall_jump_add_horizontal;
            self.velocity.0 += hadd;

            *state = PlayerState::Air(AirState {
                air_jumps: air_state.air_jumps,
                dashed: air_state.dashed,
                jump_state: JumpState::Normal,
                coyote_countdown: 0.0,
            });

            self.apply_velocity(false, |_| {});

            return;
        }

        // Stick to wall
        let offset = self.mas.depenetrate(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            &DepenetrationConfig::default(),
            &self.filter,
        );
        if let Some(hit) = self.mas.cast_move(
            &self.collider,
            self.transform.translation + offset,
            Quat::default(),
            -wstate.wall_normal * self.settings.wall_grab_max_wall_distance,
            0.01,
            &self.filter,
        ) {
            self.transform.translation += offset;
            self.transform.translation += -wstate.wall_normal * hit.distance;
        } else {
            let air_state = wstate.prev_air_state.clone();

            *state = PlayerState::Air(AirState {
                air_jumps: air_state.air_jumps,
                dashed: air_state.dashed,
                jump_state: JumpState::None,
                coyote_countdown: 0.0,
            });

            self.apply_velocity(false, |_| {});

            return;
        }
    }

    fn air_friction(&mut self) {
        let speed = self.velocity.xz().length();
        if speed < 0.01 {
            self.velocity.x = 0.0;
            self.velocity.z = 0.0;
            return;
        }

        let remove = speed.max(0.1);
        let remove = self.settings.air_friction * remove * self.dt;

        let new_speed = (speed - remove).max(0.0);

        let new_vel = self.velocity.xz().normalize() * new_speed;
        self.velocity.x = new_vel.x;
        self.velocity.z = new_vel.y;
    }

    fn air_move(&mut self) {
        if let Ok(dir) = Dir3::new(vec3(self.input.movement.x, 0.0, self.input.movement.y)) {
            let speed_towards = speed_towards_dir(vec3(self.velocity.x, 0.0, self.velocity.z), dir);
            let limit = (self.settings.air_speed - speed_towards).max(0.0);
            let to_add = limit.min(self.settings.air_accel * self.dt);
            self.velocity.0 += dir * to_add;
        }
    }

    fn ground_jump(&mut self, state: &mut PlayerState) {
        let boost = if let PlayerState::Grounded(g) = state {
            g.jump_boost
        } else {
            false
        };

        let boost = if boost {
            self.settings.slam_jump_boost
        } else {
            0.0
        };

        self.velocity.y = (2.0 * self.settings.gravity * (self.settings.jump + boost)).sqrt();
        *state = PlayerState::Air(AirState {
            jump_state: JumpState::Normal,
            ..default()
        });
    }

    fn try_wall_grab(&mut self) -> Option<Dir3> {
        let facing = Vec2::from_angle(self.pc.facing.get());
        let facing = vec3(facing.y, 0.0, facing.x);

        let hvel = vec3(self.velocity.x, 0.0, self.velocity.z);

        let mut direction: Option<(f32, Dir3, Dir3)> = None;

        for dir in [Dir3::X, Dir3::NEG_X, Dir3::Z, Dir3::NEG_Z] {
            // TODO: Replace with shape hits?
            let Some(hit) = self.mas.spatial_query.cast_shape(
                &self.collider,
                self.transform.translation,
                Quat::default(),
                dir,
                &ShapeCastConfig {
                    max_distance: self.settings.wall_grab_max_wall_distance,
                    ..default()
                },
                &self.filter,
            ) else {
                continue;
            };

            let normal = Dir3::new(hit.normal1).unwrap();

            if normal.y < self.settings.wall_grab_min_normal
                || normal.y > self.settings.wall_grab_max_normal
            {
                continue;
            }

            let speed = speed_towards_dir(hvel, normal);
            if speed > self.settings.wall_grab_max_away_velocity {
                continue;
            }

            let influence = speed_towards_dir(facing, -normal);

            if let Some((old_influence, _, _)) = direction {
                if influence > old_influence {
                    direction = Some((influence, dir, normal));
                }
            } else {
                direction = Some((influence, dir, normal));
            }
        }

        let Some((_, dir, normal)) = direction else {
            return None;
        };

        let cur_vel = self.velocity.0;

        self.velocity.0 = dir * self.settings.wall_grab_max_wall_distance;
        self.apply_velocity(false, |_| {});
        self.velocity.0 = cur_vel;

        Some(normal)
    }

    fn half_gravity(&mut self, state: &mut PlayerState) {
        if !state.grounded() {
            self.velocity.y -= self.settings.gravity * 0.5 * self.dt;
        }

        if self.velocity.y < -self.settings.terminal_velocity {
            self.velocity.y = -self.settings.terminal_velocity;
        }
    }

    fn snap_to_floor(&mut self, state: &mut PlayerState) {
        if !state.grounded() {
            return;
        }

        let offset = self.mas.depenetrate(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            &DepenetrationConfig::default(),
            &self.filter,
        );

        let Some(hit) = self.mas.cast_move(
            &self.collider,
            self.transform.translation + offset,
            Quat::default(),
            Vec3::NEG_Y * self.settings.floor_snap,
            0.01,
            &self.filter,
        ) else {
            return;
        };

        if hit.normal1.y < self.settings.min_floor_angle {
            return;
        }

        if self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }
        self.transform.translation += offset + Vec3::NEG_Y * hit.distance;
    }

    fn check_grounded(&mut self, state: &mut PlayerState) {
        let mut grounded = false;

        if self.velocity.y <= self.settings.maximum_grounded_up_velocity {
            match self.mas.spatial_query.cast_shape(
                &self.collider,
                self.transform.translation,
                Quat::default(),
                Dir3::NEG_Y,
                &ShapeCastConfig {
                    max_distance: 0.02,
                    ..default()
                },
                &self.filter,
            ) {
                Some(hit) => {
                    if hit.normal1.y > self.settings.min_floor_angle {
                        grounded = true;
                    }
                }
                None => {}
            }
        }

        if grounded && !state.grounded() {
            *state = PlayerState::Grounded(default());
        } else if !grounded && state.grounded() {
            *state = PlayerState::Air(AirState::default().with_coyote(&self.settings));
        }
    }

    fn ground_move(&mut self) {
        // Move along the ground.
        // Step up if doing so results in moving further.

        let start_pos = self.transform.translation;
        let start_vel = self.velocity.0;

        if self.apply_velocity(true, |_| {}) {
            return;
        }

        let ground_pos = self.transform.translation;
        let ground_vel = self.velocity.0;

        // Now move up, forward, and down

        self.transform.translation = start_pos;
        self.velocity.0 = start_vel;

        let step_up = self.mas.cast_move(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            Vec3::Y * self.settings.floor_snap,
            0.01,
            &self.filter,
        );

        let stepped_up = match step_up {
            None => self.settings.floor_snap,
            Some(hit) => hit.distance,
        };

        self.transform.translation.y += stepped_up;

        self.apply_velocity(true, |_| {});

        let snap_down = self.mas.cast_move(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            Vec3::NEG_Y * self.settings.step,
            0.01,
            &self.filter,
        );

        match snap_down {
            None => {
                self.transform.translation.y -= self.settings.step;
            }
            Some(hit) => {
                // If the step up would result in snapping to a non-ground surface, cancel it
                if hit.normal1.y < self.settings.min_floor_angle {
                    self.transform.translation = ground_pos;
                    **self.velocity = ground_vel;
                    return;
                }

                self.transform.translation.y -= hit.distance;
            }
        }

        let step_pos = self.transform.translation;

        let dist_grounded = (ground_pos - start_pos).length_squared();
        let dist_step = (step_pos - start_pos).length_squared();

        if dist_grounded > dist_step {
            self.transform.translation = ground_pos;
            **self.velocity = ground_vel;
        } else {
            self.velocity.y = ground_vel.y;
        }
    }

    /// Returns true if the velocity was applied unimpeded
    fn apply_velocity(
        &mut self,
        grounded: bool,
        mut hit_callback: impl FnMut(&mut MoveAndSlideHitData),
    ) -> bool {
        let mut unimpeded = true;

        let MoveAndSlideOutput {
            position,
            projected_velocity,
        } = self.mas.move_and_slide(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            self.velocity.0,
            self.delta,
            &MoveAndSlideConfig {
                planes: if grounded { vec![Dir3::Y] } else { vec![] },
                ..default()
            },
            &self.filter,
            |mut hit| {
                unimpeded = false;

                hit_callback(&mut hit);

                MoveAndSlideHitResponse::Accept
            },
        );

        self.transform.translation = position;
        self.velocity.0 = projected_velocity;

        unimpeded
    }
}

fn speed_towards_dir(speed: Vec3, dir: Dir3) -> f32 {
    let angle = speed.angle_between(dir.as_vec3());

    if angle <= PI / 2.0 {
        let vel_towards_dir = speed.project_onto(dir.as_vec3());
        vel_towards_dir.length()
    } else {
        0.0
    }
}

#[derive(Message, Clone, PartialEq)]
pub enum PlayerControllerMessage {
    GroundJump,
    CoyoteTimeJump,
    CoyoteFrictionJump,
    AirJump,
    Slam(Vec3),
}
