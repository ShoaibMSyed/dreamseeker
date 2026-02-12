use bevy::prelude::*;
use dreamseeker_util::construct::Make;

// pub(super) fn plugin(app: &mut App) {
//     let _ = app;
// }

#[derive(Component, Reflect, Default)]
#[require(Name::new("Sword"))]
pub struct Sword;

impl Sword {
    pub fn bundle() -> impl Bundle {
        (Self, Make(Self::make))
    }

    fn make(assets: Res<AssetServer>) -> Result<impl Bundle + use<>> {
        Ok(SceneRoot(assets.load("sword.glb#Scene0")))
    }
}
