use avian3d::{
    PhysicsPlugins,
    prelude::{DebugRender, PhysicsDebugPlugin, PhysicsGizmos},
};
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_framepace::FramepacePlugin;
use bevy_skein::SkeinPlugin;
use dreamseeker_util::DreamSeekerUtil;

use self::player::{Player, camera::PlayerCamera};

mod input;
mod player;
mod trigger;
mod util;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            DreamSeeker,
            SkeinPlugin::default(),
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
        ));

        *app.world_mut()
            .resource_mut::<GizmoConfigStore>()
            .config_mut::<PhysicsGizmos>()
            .1 = PhysicsGizmos::none();

        app.add_systems(Startup, setup)
            .add_systems(Update, capture_mouse);
    }
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

    // Spawn Terrain

    cmd.spawn(SceneRoot(assets.load("level.glb#Scene0")));
}

fn capture_mouse(
    mut captured: Local<bool>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        *captured = !*captured;

        if *captured {
            cursor.grab_mode = CursorGrabMode::Confined;
            cursor.visible = false;
        } else {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}
