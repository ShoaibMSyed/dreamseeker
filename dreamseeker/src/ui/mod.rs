use bevy::prelude::*;

pub mod item;
pub mod pause;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((self::item::plugin, self::pause::plugin));
}

#[derive(Component, Reflect, Default)]
pub struct Screen;
