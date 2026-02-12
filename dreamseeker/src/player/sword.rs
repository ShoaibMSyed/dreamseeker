use avian3d::prelude::*;
use bevy::prelude::*;
use dreamseeker_util::construct::Make;

use crate::collision::GameLayer;

// pub(super) fn plugin(app: &mut App) {
//     let _ = app;
// }

#[derive(Component, Reflect, Default)]
#[require(
    Name::new("Sword"),
    CollisionLayers::new(GameLayer::Sword, GameLayer::Bouncy),
    CollisionEventsEnabled,
    Collider::compound(vec![(
        vec3(0.0, 0.7, 0.0),
        Quat::default(),
        Collider::cuboid(0.2, 1.4, 0.2),
    )]),
    Sensor,
)]
pub struct Sword;

impl Sword {
    pub fn bundle() -> impl Bundle {
        (Self, Make(Self::make))
    }

    fn make(assets: Res<AssetServer>) -> Result<impl Bundle + use<>> {
        Ok(SceneRoot(assets.load("sword.glb#Scene0")))
    }
}
