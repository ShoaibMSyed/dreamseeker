use bevy::prelude::*;
use bevy_enhanced_input::prelude::InputContextAppExt;

use crate::player::{Player, camera::PlayerCamera};

pub(super) fn plugin(app: &mut App) {
    app
        .add_input_context_to::<FixedPreUpdate, Player>()
        .add_input_context::<PlayerCamera>();
}

pub mod camera {
    use bevy::prelude::*;
    use bevy_enhanced_input::prelude::{*, Press};

    use crate::player::camera::PlayerCamera;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct CenterCamera;


    #[derive(InputAction)]
    #[action_output(Vec2)]
    pub struct MoveCamera;

    pub fn actions() -> impl Bundle {
        actions!(PlayerCamera[
            (
                Action::<CenterCamera>::new(),
                Press::default(),
                bindings![
                    KeyCode::ShiftLeft,
                    GamepadButton::LeftTrigger2,
                    (
                        GamepadAxis::LeftZ,
                        Clamp::pos(),
                    )
                ],
            ),
            (
                Action::<MoveCamera>::new(),
                Bindings::spawn((
                    Spawn((
                        Binding::mouse_motion(),
                        Scale::splat(0.2),
                        Negate::all(),
                    )),
                    Axial::right_stick()
                        .with(DeadZone::default()),
                )),
            ),
        ])
    }
}

pub mod player {
    use bevy::prelude::*;
    use bevy_enhanced_input::prelude::*;

    use crate::player::Player;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct Jump;

    #[derive(InputAction)]
    #[action_output(Vec2)]
    pub struct Move;

    #[derive(InputAction)]
    #[action_output(bool)]
    pub struct Slide;

    pub fn actions() -> impl Bundle {
        actions!(Player[
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space, GamepadButton::East],
            ),
            (
                Action::<Move>::new(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick()
                        .with(DeadZone::new(DeadZoneKind::Axial)),
                )),
            ),
            (
                Action::<Slide>::new(),
                bindings![MouseButton::Right, GamepadButton::South],
            ),
        ])
    }
}