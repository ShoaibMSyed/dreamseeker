use bevy::prelude::*;

use super::Screen;

pub(super) fn plugin(_app: &mut App) {}

#[derive(Component)]
#[require(Screen)]
pub struct InfoScreen;

impl InfoScreen {
    pub fn bundle(text: String) -> impl Bundle {
        let text = (
            Text::new(text),
            TextFont::from_font_size(20.0),
            TextLayout::new_with_justify(Justify::Center),
        );

        (
            Self,
            Node {
                width: percent(100),
                height: percent(50),
                align_self: AlignSelf::End,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::top(px(1)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.5)),
            Children::spawn_one(text),
        )
    }
}
