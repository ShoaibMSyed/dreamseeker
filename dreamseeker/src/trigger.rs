use avian3d::prelude::*;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    collision::GameLayer,
    player::{Die, Player},
    ui::screen::{ScreenCommandsExt, ScreenStack, info::InfoScreen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, check_collisions);
}

fn check_collisions(
    mut reader: MessageReader<CollisionStart>,
    q_tele: Query<&TriggerTeleport>,
    mut cmd: Commands,
) {
    for col in reader.read() {
        let e1 = col.body1.unwrap_or(col.collider1);
        let e2 = col.body2.unwrap_or(col.collider2);

        if q_tele.contains(e1) {
            cmd.run_system_cached_with(TriggerTeleport::on_collision, (e1, e2));
        }

        if q_tele.contains(e2) {
            cmd.run_system_cached_with(TriggerTeleport::on_collision, (e2, e1));
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
#[require(Transform, Sensor, CollisionEventsEnabled)]
pub struct TriggerTeleport(pub Vec3);

impl Default for TriggerTeleport {
    fn default() -> Self {
        Self(vec3(0.0, 2.0, 0.0))
    }
}

impl TriggerTeleport {
    fn on_collision(
        In((this, entity)): In<(Entity, Entity)>,
        q: Query<&Self>,
        mut player: Query<&mut Transform, With<Player>>,
    ) {
        let this = q.get(this).unwrap();
        let Ok(mut transform) = player.get_mut(entity) else {
            return;
        };

        transform.translation = this.0;
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
#[require(
    Transform,
    Sensor,
    CollisionEventsEnabled,
    CollisionLayers::new(GameLayer::Sensor, LayerMask::ALL)
)]
#[component(on_add)]
pub struct DeathTrigger;

impl DeathTrigger {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .observe(Self::on_collision);
    }

    fn on_collision(event: On<CollisionStart>, mut cmd: Commands) {
        cmd.trigger(Die(event.collider2));
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
#[require(Transform)]
pub struct RespawnPoint;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
#[require(
    Transform,
    Sensor,
    CollisionEventsEnabled,
    CollisionLayers::new(GameLayer::Sensor, LayerMask::ALL)
)]
#[component(on_add)]
pub struct RespawnTrigger;

impl RespawnTrigger {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .observe(Self::on_collision);
    }

    fn on_collision(
        event: On<CollisionStart>,
        parent: Query<&ChildOf>,
        children: Query<&Children>,
        point: Query<(&RespawnPoint, &GlobalTransform)>,
        mut player: Query<&mut Player>,
    ) -> Result {
        let parent = parent.get(event.collider1)?.0;
        for desc in children.iter_descendants(parent) {
            if let Ok((_, transform)) = point.get(desc) {
                if let Ok(mut player) = player.get_mut(event.collider2) {
                    player.respawn = Some(transform.translation());
                }
            }
        }

        Ok(())
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Transform)]
#[component(on_add)]
pub struct InitialSpawn;

impl InitialSpawn {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let pos = world.get::<Transform>(ctx.entity).unwrap().translation;

        world
            .commands()
            .spawn((Player::bundle(), Transform::from_translation(pos + Vec3::Y)));
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Transform)]
pub struct MainSpawn;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
#[require(
    Transform,
    Sensor,
    CollisionEventsEnabled,
    CollisionLayers::new(GameLayer::Sensor, LayerMask::ALL),
    RigidBody::Static,
    Collider::compound(vec![
        (
            vec3(0.0, 1.0, 0.0),
            Quat::default(),
            Collider::sphere(1.0),
        )
    ]),
)]
#[component(on_add)]
pub struct InfoTrigger(pub String);

impl InfoTrigger {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let sign = world.load_asset("sign.glb#Scene0");

        world
            .commands()
            .entity(ctx.entity)
            .insert(SceneRoot(sign))
            .observe(Self::on_enter)
            .observe(Self::on_exit);
    }

    fn on_enter(
        event: On<CollisionStart>,
        info: Query<&InfoTrigger>,
        player: Query<&Player>,
        mut cmd: Commands,
    ) -> Result {
        if player.contains(event.collider2) {
            let info = info.get(event.collider1)?;
            cmd.push_screen(InfoScreen::bundle(info.0.clone()));
        }
        Ok(())
    }

    fn on_exit(
        event: On<CollisionEnd>,
        player: Query<&Player>,
        q: Query<&InfoScreen>,
        screen: Res<ScreenStack>,
        mut cmd: Commands,
    ) {
        if player.contains(event.collider2)
            && let Some(cur) = screen.current()
            && q.contains(cur)
        {
            cmd.pop_screen();
        }
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
#[require(Transform, Sensor, CollisionEventsEnabled)]
#[component(on_add)]
pub struct EnableMainSpawn(pub String);

impl EnableMainSpawn {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).observe(Self::on_enter);
    }

    fn on_enter(event: On<CollisionStart>, mut player: Query<&mut Player>) -> Result {
        if let Ok(mut player) = player.get_mut(event.collider2) {
            player.main_spawn = true;
        }
        Ok(())
    }
}
