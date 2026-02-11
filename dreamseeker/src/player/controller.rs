use std::{f32::consts::PI, time::Duration};

use avian3d::{character_controller::move_and_slide::DepenetrationConfig, prelude::*};
use bevy::{ecs::query::QueryData, prelude::*};
use bevy_enhanced_input::prelude::*;

use crate::{
    input::player::{Jump, Move, Slide}, player::{PLAYER_HEIGHT, PLAYER_WIDTH}, util::angle::Angle
};

use super::{PlayerModel, camera::PlayerCamera};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (PlayerInput::gather, PlayerController::step, PlayerController::set_collider).chain(),
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

    pub coyote_time: f32,
    pub terminal_velocity: f32,

    pub air_jumps: u8,
    pub air_jump_forward_boost: f32,
    
    
    pub run_speed: f32,

    pub slide_enabled: bool,
    pub slide_speed: f32,
    pub slide_time: f32,
}

impl Default for PlayerControllerSettings {
    fn default() -> Self {
        Self {
            gravity: 20.0,
            floor_snap: 0.25,
            step: 0.33,
            jump: 1.0,
            min_floor_angle: 0.7,
            maximum_grounded_up_velocity: 6.0,

            coyote_time: (1.0 / 64.0) * 5.0,
            terminal_velocity: 10.0,

            air_jumps: 2,
            air_jump_forward_boost: 4.5,
            
            run_speed: 5.5,

            slide_enabled: true,
            slide_speed: 7.5,
            slide_time: 0.7,
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct PlayerInput {
    pub movement: Vec2,
    pub jump: ActionEvents,
    pub slide: ActionEvents,
}

impl PlayerInput {
    fn gather(
        mut pi: Single<(&mut PlayerController, &mut PlayerInput, &PlayerState)>,
        camera: Single<&PlayerCamera>,
        jump: Single<&ActionEvents, With<Action<Jump>>>,
        slide: Single<&ActionEvents, With<Action<Slide>>>,
        movement: Single<&Action<Move>>,
    ) {
        let dir = Vec3::new(movement.x, 0.0, -movement.y)
            .normalize_or_zero()
            .rotate_y((camera.rotation + Angle::new(std::f32::consts::PI)).get());
        pi.1.movement.x = dir.x;
        pi.1.movement.y = dir.z;

        if pi.1.movement.length_squared() > 0.0
            && !pi.2.facing_locked()
        {
            let angle = Vec2::new(pi.1.movement.y, pi.1.movement.x).to_angle();
            pi.0.facing = Angle::new(angle);
        }

        pi.1.jump = **jump;
        pi.1.slide = **slide;
    }
}

#[derive(Reflect, Default, PartialEq, Eq)]
pub enum JumpState {
    #[default]
    None,
    Normal,
    Halved,
}

#[derive(Reflect, Default)]
pub struct GroundedState;

#[derive(Reflect, Default)]
pub struct AirState {
    pub air_jumps: u8,
    pub jump_state: JumpState,
    pub coyote_countdown: f32,
}

impl AirState {
    fn with_coyote(mut self, settings: &PlayerControllerSettings) -> Self {
        self.coyote_countdown = settings.coyote_time;
        self
    }
}

#[derive(Reflect, Default)]
pub struct SlidingState {
    direction: Vec2,
    timer: f32,
}

#[derive(Component, Reflect)]
pub enum PlayerState {
    Grounded(GroundedState),
    Air(AirState),
    Sliding(SlidingState),
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
        matches!(self, Self::Sliding(_))
    }
}

#[derive(Component, Reflect, Default)]
#[require(
    Collider,
    RigidBody::Kinematic,
    TransformInterpolation,
    CustomPositionIntegration,
    PlayerInput,
    PlayerState,
    PlayerControllerSettings,
)]
pub struct PlayerController {
    pub facing: Angle,
}

impl PlayerController {
    fn step(query: Query<MovementData>, mut mas: MoveAndSlide, time: Res<Time>) {
        for move_data in query {
            let mut mover = Mover::new(move_data, &mut mas, time.delta());

            mover.step();
        }
    }

    fn set_collider(
        mut sliding: Local<bool>,
        mut player: Single<(&PlayerState, &mut Collider, &mut Transform, &mut PlayerController), Changed<PlayerState>>,
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
    state: &'static mut PlayerState,
    input: &'static PlayerInput,
    settings: &'static PlayerControllerSettings,
    collider: &'static Collider,
}

#[derive(Deref, DerefMut)]
struct Mover<'a, 'w1, 's1, 'w2, 's2> {
    #[deref]
    data: MovementDataItem<'w1, 's1>,
    mas: &'a mut MoveAndSlide<'w2, 's2>,
    delta: Duration,
    dt: f32,
}

impl<'a, 'w, 's, 'w2, 's2> Mover<'a, 'w, 's, 'w2, 's2> {
    fn new(
        data: MovementDataItem<'w, 's>,
        mas: &'a mut MoveAndSlide<'w2, 's2>,
        delta: Duration,
    ) -> Self {
        Self {
            data,
            mas,
            dt: delta.as_secs_f32(),
            delta,
        }
    }

    fn step(&mut self) {
        self.half_gravity();

        self.update_state();

        self.snap_to_floor();
        self.half_gravity();
        self.check_grounded();
    }

    fn update_state(&mut self) {
        match &*self.data.state {
            PlayerState::Grounded(_) => self.update_grounded(),
            PlayerState::Air(_) => self.update_air(),
            PlayerState::Sliding(_) => self.update_sliding(),
        }
    }

    fn update_grounded(&mut self) {
        let PlayerState::Grounded(GroundedState) = &mut *self.data.state
        else { return };

        self.velocity.x = self.input.movement.x * self.settings.run_speed;
        self.velocity.z = self.input.movement.y * self.settings.run_speed;

        self.ground_move();

        if self.input.jump.contains(ActionEvents::START) {
            self.ground_jump();
        } else if self.input.slide.contains(ActionEvents::START)
            && self.settings.slide_enabled
        {
            let dir = Vec2::from_angle(self.pc.facing.get());
            *self.state = PlayerState::Sliding(SlidingState {
                direction: Vec2::new(dir.y, dir.x),
                timer: 0.0,
            });
        }
    }

    fn update_air(&mut self) {
        let PlayerState::Air(state) = &mut *self.data.state
        else { return };

        state.coyote_countdown = (state.coyote_countdown - self.dt).max(0.0);

        if state.jump_state == JumpState::Normal
            && self.data.velocity.y > 0.0
            && !self.data.input.jump.contains(ActionEvents::FIRE)
        {
            self.data.velocity.y /= 2.0;
            state.jump_state = JumpState::Halved;
        }

        if state.jump_state != JumpState::None && self.data.velocity.y <= 0.0 {
            state.jump_state = JumpState::None;
        }

        if state.air_jumps < self.data.settings.air_jumps
            && self.data.input.jump.contains(ActionEvents::START)
        {
            if state.coyote_countdown <= 0.0 {
                // Air Jump
                self.data.velocity.y = (2.0 * self.data.settings.gravity * self.data.settings.jump).sqrt();
                state.air_jumps += 1;
                state.jump_state = JumpState::Normal;

                // Forward Boost

                if let Ok(dir) = Dir3::new(vec3(self.data.input.movement.x, 0.0, self.data.input.movement.y)) {
                    let hvel = vec3(self.data.velocity.x, 0.0, self.data.velocity.z);
                    let speed_towards_dir = {
                        let angle = hvel.angle_between(dir.as_vec3());

                        if angle <= PI / 2.0 {
                            let vel_towards_dir = hvel.project_onto(dir.as_vec3());
                            vel_towards_dir.length()
                        } else {
                            0.0
                        }
                    };

                    let boost = (self.settings.air_jump_forward_boost - speed_towards_dir).max(0.0);
                    self.data.velocity.0 += dir * boost;
                }
            } else {
                // Coyote Time
                self.ground_jump();
            }
        }

        self.apply_velocity();
    }

    fn update_sliding(&mut self) {
        let PlayerState::Sliding(state) = &mut *self.data.state
        else { return };

        state.timer += self.dt;

        self.data.velocity.x = state.direction.x * self.data.settings.slide_speed;
        self.data.velocity.z = state.direction.y * self.data.settings.slide_speed;

        if state.timer >= self.data.settings.slide_time {
            *self.state = PlayerState::Grounded(GroundedState);
        }

        if self.input.jump.contains(ActionEvents::START) {
            self.ground_jump();
        }

        self.ground_move();
    }

    fn ground_jump(&mut self) {
        self.velocity.y = (2.0 * self.settings.gravity * self.settings.jump).sqrt();
        *self.state = PlayerState::Air(AirState {
            jump_state: JumpState::Normal,
            ..default()
        });
    }

    fn half_gravity(&mut self) {
        if !self.state.grounded() {
            self.velocity.y -= self.settings.gravity * 0.5 * self.dt;
        }

        if self.velocity.y < -self.settings.terminal_velocity {
            self.velocity.y = -self.settings.terminal_velocity;
        }
    }

    fn snap_to_floor(&mut self) {
        if !self.state.grounded() {
            return;
        }

        let offset = self.mas.depenetrate(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            &DepenetrationConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([self.entity]),
        );

        let Some(hit) = self.mas.cast_move(
            &self.collider,
            self.transform.translation + offset,
            Quat::default(),
            Vec3::NEG_Y * self.settings.floor_snap,
            0.01,
            &SpatialQueryFilter::from_excluded_entities([self.entity]),
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

    fn check_grounded(&mut self) {
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
                &SpatialQueryFilter::from_excluded_entities([self.entity]),
            ) {
                Some(hit) => {
                    if hit.normal1.y > self.settings.min_floor_angle {
                        grounded = true;
                    }
                }
                None => {}
            }
        }

        if grounded && !self.state.grounded() {
            *self.state = PlayerState::Grounded(GroundedState);
        } else if !grounded && self.state.grounded() {
            *self.state = PlayerState::Air(AirState::default().with_coyote(&self.settings));
        }
    }

    fn ground_move(&mut self) {
        // Move along the ground.
        // Step up if doing so results in moving further.

        let start_pos = self.transform.translation;
        let start_vel = self.velocity.0;

        if self.apply_velocity() {
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
            &SpatialQueryFilter::from_excluded_entities([self.entity]),
        );

        let stepped_up = match step_up {
            None => self.settings.floor_snap,
            Some(hit) => hit.distance,
        };

        self.transform.translation.y += stepped_up;

        self.apply_velocity();

        let snap_down = self.mas.cast_move(
            &self.collider,
            self.transform.translation,
            Quat::default(),
            Vec3::NEG_Y * self.settings.step,
            0.01,
            &SpatialQueryFilter::from_excluded_entities([self.entity]),
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
    fn apply_velocity(&mut self) -> bool {
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
                planes: if self.state.grounded() {
                    vec![Dir3::Y]
                } else {
                    vec![]
                },
                ..default()
            },
            &SpatialQueryFilter::from_excluded_entities([self.entity]),
            |_hit| {
                unimpeded = false;

                MoveAndSlideHitResponse::Accept
            },
        );

        self.transform.translation = position;
        self.velocity.0 = projected_velocity;

        unimpeded
    }
}
