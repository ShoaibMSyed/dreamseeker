use bevy::prelude::*;

pub mod add_asset;
pub mod construct;
pub mod observers;

pub struct DreamSeekerUtil;

impl Plugin for DreamSeekerUtil {
    fn build(&self, app: &mut App) {
        app.add_plugins(self::observers::plugin);
    }
}