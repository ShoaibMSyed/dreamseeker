use avian3d::{
    PhysicsPlugins,
    prelude::{DebugRender, PhysicsDebugPlugin, PhysicsGizmos},
};
use bevy::{
    input::common_conditions::input_toggle_active,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::{EnhancedInputPlugin, prelude::ActionSources};
use bevy_flurx::FlurxPlugin;
use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_skein::SkeinPlugin;
use dreamseeker_util::DreamSeekerUtil;

use self::{
    enemy::Enemy,
    player::{Player, camera::PlayerCamera},
};

mod collision;
mod enemy;
mod input;
mod player;
mod trigger;
mod ui;
mod util;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            FlurxPlugin,
            DreamSeeker,
            SkeinPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::KeyG)),
        ))
        .run();
}

struct DreamSeeker;

impl Plugin for DreamSeeker {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EnhancedInputPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
            DreamSeekerUtil,
            self::input::plugin,
            self::player::plugin,
            self::trigger::plugin,
            self::ui::plugin,
        ));

        *app.world_mut()
            .resource_mut::<GizmoConfigStore>()
            .config_mut::<PhysicsGizmos>()
            .1 = PhysicsGizmos::none();

        app.init_resource::<Sounds>()
            .init_state::<GameState>()
            .add_systems(Startup, setup)
            .add_systems(Update, capture_mouse);
    }
}

#[derive(States, Reflect, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    InGame,
    Cutscene,
    Paused,
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.insert_resource(GlobalAmbientLight {
        brightness: 200.0,
        ..default()
    });

    cmd.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-10.0, 10.0, 0.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    cmd.spawn(PlayerCamera::bundle());

    cmd.spawn((
        Player::bundle(),
        DebugRender::none(),
        Transform::from_xyz(0.0, 3.0, 0.0),
    ));

    cmd.spawn((Enemy::bundle(), Transform::from_xyz(-10.0, 0.0, -10.0)));

    // Spawn Terrain

    cmd.spawn((
        Name::new("Scene"),
        SceneRoot(assets.load("level.glb#Scene0")),
    ));
}

fn capture_mouse(
    mut not_first: Local<bool>,
    mut captured: Local<bool>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut actions: ResMut<ActionSources>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyG) || !*not_first {
        *not_first = true;
        *captured = !*captured;

        if *captured {
            cursor.grab_mode = CursorGrabMode::Confined;
            cursor.visible = false;
            actions.keyboard = true;
            actions.mouse_buttons = true;
            actions.mouse_motion = true;
            actions.mouse_wheel = true;
        } else {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
            actions.keyboard = false;
            actions.mouse_buttons = false;
            actions.mouse_motion = false;
            actions.mouse_wheel = false;
        }
    }
}

macro_rules! sounds {
    ($($name:ident),* $(,)?) => {
        #[derive(Resource)]
        pub struct Sounds {
            $(pub $name: Handle<AudioSource>,)*
        }

        impl FromWorld for Sounds {
            fn from_world(world: &mut World) -> Self {
                Sounds {
                    $($name: world.load_asset(format!("{}.ogg", stringify!($name))),)*
                }
            }
        }
    }
}

sounds!(
    air_jump,
    chest_open,
    coyote_friction_jump,
    coyote_time_jump,
    footstep,
    item_get,
    jump,
    sword_hit,
    sword_swing
);
