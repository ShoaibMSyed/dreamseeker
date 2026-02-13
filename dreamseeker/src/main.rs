use avian3d::{
    PhysicsPlugins,
    prelude::{DebugRender, PhysicsDebugPlugin, PhysicsGizmos},
};
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::EnhancedInputPlugin;
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
            WorldInspectorPlugin::new().run_if(in_state(GameState::Paused)),
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
            .add_systems(Startup, setup);
    }
}

#[derive(States, Reflect, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    InGame,
    Cutscene,
    Paused,
}

fn setup(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
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

    cursor.grab_mode = CursorGrabMode::Confined;
    cursor.visible = false;
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
