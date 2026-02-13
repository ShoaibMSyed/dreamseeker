use avian3d::prelude::*;
use bevy::prelude::*;
use dreamseeker_util::construct::Make;

use crate::collision::GameLayer;

#[derive(Component, Reflect, Default)]
#[require(
    CollisionLayers::new(GameLayer::Attackable, LayerMask::ALL),
    RigidBody::Kinematic,
    Collider::sphere(1.0)
)]
pub struct Enemy;

impl Enemy {
    pub fn bundle() -> impl Bundle {
        (Self, Make(Self::make))
    }

    fn make(
        mut meshes: ResMut<Assets<Mesh>>,
        mut mats: ResMut<Assets<StandardMaterial>>,
    ) -> Result<impl Bundle + use<>> {
        Ok((
            Mesh3d(meshes.add(Sphere::new(1.0))),
            MeshMaterial3d(mats.add(Color::linear_rgb(0.8, 0.4, 0.1))),
        ))
    }
}
