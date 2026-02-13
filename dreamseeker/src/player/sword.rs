use avian3d::prelude::*;
use bevy::prelude::*;
use dreamseeker_util::construct::Make;

use crate::collision::GameLayer;

use super::item::{Item, PlayerItems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, Sword::update);
}

#[derive(Component, Reflect, Default)]
#[require(
    Name::new("Sword"),
    CollisionLayers::new(GameLayer::Sword, GameLayer::Attackable),
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

    fn update(
        player: Single<&PlayerItems, Changed<PlayerItems>>,
        mut sword: Single<&mut Visibility, With<Sword>>,
    ) {
        **sword = match player.contains(&Item::Sword) {
            false => Visibility::Hidden,
            true => Visibility::Inherited,
        };
    }
}
