use avian3d::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    platform::collections::HashSet,
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_flurx::{
    action::{once, wait},
    prelude::Reactor,
    task::ReactorTask,
};

use crate::{
    GameState, Sounds,
    collision::GameLayer,
    ui::item::{ScreenClose, item_description},
};

use super::{Player, controller::PlayerControllerSettings};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(Chest::on_hit)
        .add_systems(Update, PlayerItems::on_update);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(
    Name::new("Token"),
    Item,
    RigidBody::Static,
    Collider::sphere(0.5),
    CollisionLayers::new(GameLayer::Attackable, LayerMask::ALL),
    CollisionEventsEnabled
)]
#[component(on_add)]
pub struct Token;

impl Token {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let scene = SceneRoot(world.load_asset("token.glb#Scene0"));
        world
            .commands()
            .entity(ctx.entity)
            .observe(Self::on_hit)
            .insert(scene);
    }

    fn on_hit(event: On<CollisionStart>, mut player: Query<&mut Player>, mut cmd: Commands) {
        let Ok(mut player) = player.get_mut(event.collider2) else {
            return;
        };

        player.dream_tokens += 1;

        cmd.entity(event.collider1).despawn();
    }
}

#[derive(Component, Reflect, Default, Deref, DerefMut)]
pub struct PlayerItems(pub HashSet<Item>);

impl PlayerItems {
    fn on_update(
        mut player: Single<(&mut PlayerControllerSettings, &PlayerItems), Changed<PlayerItems>>,
    ) {
        player.0.air_jumps = 0;
        for cloud in [Item::Cloud1, Item::Cloud2, Item::Cloud3] {
            if player.1.contains(&cloud) {
                player.0.air_jumps += 1;
            }
        }
        player.0.dash_enabled = player.1.contains(&Item::Rocket);
        player.0.slide_enabled = player.1.contains(&Item::Ice);
        player.0.slam_enabled = player.1.contains(&Item::Anvil);
        player.0.wall_grab_enabled = player.1.contains(&Item::Scroll);
    }
}

#[derive(Component, Reflect, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[reflect(Component, Default)]
pub enum Item {
    #[default]
    Cloud1,
    Cloud2,
    Cloud3,
    Rocket,
    Ice,
    Anvil,
    Scroll,
    Sword,
}

impl Item {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cloud1 | Self::Cloud2 | Self::Cloud3 => "Cloud",
            Self::Rocket => "Rocket",
            Self::Ice => "Ice",
            Self::Anvil => "Anvil",
            Self::Scroll => "Ninja Scroll",
            Self::Sword => "Sword",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Cloud1 | Self::Cloud2 | Self::Cloud3 => {
                "You gained an additional jump!\nYou can jump again while you are in the air"
            }
            Self::Rocket => "You can air dash!\nPress Right-Click / X in the air to dash forward.",
            Self::Ice => "You can slide!\nPress Shift / A to slide along the ground",
            Self::Anvil => "You can slam!\nPress Shift / A in the air to slam into the ground",
            Self::Scroll => {
                "You can grab on to walls!\nHold Control / Right Trigger to grab a wall"
            }
            Self::Sword => "You can attack!\nPress Left Click / X in the air to use your sword",
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Name::new("Chest"), Item)]
#[component(on_add)]
pub struct Chest {
    open: AnimationNodeIndex,
    aplayer: Option<Entity>,
}

impl Chest {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let scene = SceneRoot(world.load_asset("chest.glb#Scene0"));
        world
            .commands()
            .entity(ctx.entity)
            .observe(Self::on_load)
            .insert(scene);
    }

    fn on_load(
        event: On<SceneInstanceReady>,
        children: Query<&Children>,
        col: Query<&CollisionEventsEnabled>,
        aplayer: Query<&AnimationPlayer>,
        assets: Res<AssetServer>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
        mut cmd: Commands,
    ) {
        let mut graph = AnimationGraph::new();
        let open = graph.add_clip(assets.load("chest.glb#Animation0"), 1.0, graph.root);
        let graph = graphs.add(graph);

        let mut ap = None;

        for entity in children.iter_descendants(event.entity) {
            if col.contains(entity) {
                cmd.entity(entity)
                    .insert(CollisionLayers::new(GameLayer::Attackable, LayerMask::ALL));
            }

            if aplayer.contains(entity) {
                ap = Some(entity);
                cmd.entity(entity)
                    .insert(AnimationGraphHandle(graph.clone()));
            }
        }

        cmd.entity(event.entity).insert(Chest { open, aplayer: ap });
    }

    fn on_hit(
        event: On<CollisionStart>,
        parent: Query<&ChildOf>,
        q_chest: Query<(&Chest, &Item)>,
        player: Query<&Player>,
        mut cmd: Commands,
    ) {
        for entity in parent.iter_ancestors(event.collider1) {
            if !player.contains(event.collider2) {
                continue;
            }

            let Ok((_, item)) = q_chest.get(entity) else {
                continue;
            };
            let item = item.clone();

            cmd.spawn(Reactor::schedule(move |task| {
                item_get_cutscene(task, entity, item)
            }));
        }
    }
}

async fn item_get_cutscene(task: ReactorTask, chest: Entity, item: Item) {
    task.will(
        PreUpdate,
        once::run(
            move |mut state: ResMut<NextState<GameState>>,
                  q_chest: Query<&Chest>,
                  mut aplayer: Query<&mut AnimationPlayer>,
                  mut cmd: Commands,
                  sounds: Res<Sounds>|
                  -> Option<()> {
                state.set(GameState::Cutscene);
                let chest = q_chest.get(chest).ok()?;

                let mut aplayer = chest.aplayer.and_then(|ap| aplayer.get_mut(ap).ok())?;
                aplayer.play(chest.open);

                cmd.spawn((
                    AudioPlayer::new(sounds.chest_open.clone()),
                    PlaybackSettings::DESPAWN,
                ));

                Some(())
            },
        ),
    )
    .await;

    task.will(
        Update,
        wait::until(
            move |q_chest: Query<&Chest>, aplayer: Query<&AnimationPlayer>| {
                let Ok(chest) = q_chest.get(chest) else {
                    return true;
                };
                let Some(ap) = chest.aplayer else { return true };
                let Ok(aplayer) = aplayer.get(ap) else {
                    return true;
                };

                let Some(anim) = aplayer.animation(chest.open) else {
                    return true;
                };

                anim.is_finished()
            },
        ),
    )
    .await;

    task.will(
        PreUpdate,
        once::run(move |mut cmd: Commands, sounds: Res<Sounds>| {
            cmd.spawn(item_description(item));

            cmd.spawn((
                AudioPlayer::new(sounds.item_get.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }),
    )
    .await;

    task.will(Update, wait::message::comes::<ScreenClose>())
        .await;

    task.will(
        PreUpdate,
        once::run(
            move |mut state: ResMut<NextState<GameState>>,
                  mut player: Single<&mut PlayerItems>,
                  mut cmd: Commands| {
                cmd.entity(chest).despawn();
                state.set(GameState::InGame);
                player.insert(item);
            },
        ),
    )
    .await;
}
