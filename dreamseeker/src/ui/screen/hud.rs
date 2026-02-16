use bevy::prelude::*;

use crate::player::Player;

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, TokenCounter::update);
}

#[derive(Component)]
#[require(Screen)]
pub struct HudScreen;

impl HudScreen {
    pub fn bundle() -> impl Bundle {
        let tokens = (
            Text::new("0 tokens"),
            TextFont::from_font_size(20.0),
            TokenCounter,
        );

        (
            Self,
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(20)),
                ..default()
            },
            children![tokens],
        )
    }
}

#[derive(Component)]
struct TokenCounter;

impl TokenCounter {
    fn update(text: Query<(&mut Text, &TokenCounter)>, player: Single<&Player, Changed<Player>>) {
        for (mut text, _) in text {
            text.0 = format!("{} tokens", player.dream_tokens);
        }
    }
}
