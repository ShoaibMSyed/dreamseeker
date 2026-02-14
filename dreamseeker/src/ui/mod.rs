use bevy::prelude::*;

pub mod screen;

pub use self::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(self::screen::plugin);
}
