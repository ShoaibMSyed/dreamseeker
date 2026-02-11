use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::Player;

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
#[require(
    Transform,
    Sensor,
    CollisionEventsEnabled,
)]
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
        let Ok(mut transform) = player.get_mut(entity)
        else { return };

        transform.translation = this.0;
    }
}