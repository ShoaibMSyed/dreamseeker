use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::Fire;
use dreamseeker_util::{construct::Make, observers};

use crate::{
    GameState,
    input::ui::{Respawn, actions},
    player::{
        Player,
        item::{Item, PlayerItems},
    },
    trigger::MainSpawn,
};

use super::{
    Screen, ScreenHidden, ScreenShown,
    systems::{pop_screen, push_screen},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Paused), push_screen(PauseScreen::bundle))
        .add_systems(OnExit(GameState::Paused), pop_screen());
}

#[derive(Component, Reflect)]
#[require(Screen)]
pub struct PauseScreen;

impl PauseScreen {
    pub fn bundle() -> impl Bundle {
        let list = (
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            Make(Self::make_entries),
        );

        let body = (
            Node {
                width: percent(100),
                padding: UiRect::horizontal(percent(20)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            children![list],
        );

        let title = (
            Text::new("Items"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
        );

        let foot = (
            Text::new("Hold Space / A to respawn in the hub"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextLayout::new_with_justify(Justify::Center),
        );

        (
            PauseScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.5)),
            actions(),
            observers![Self::on_respawn, Self::on_shown, Self::on_hidden],
            children![title, body, foot],
        )
    }

    fn make_entries(items: Single<&PlayerItems>) -> Result<impl Bundle + use<>> {
        let mut items = items.iter().copied().collect::<Vec<Item>>();
        items.sort();

        Ok(Children::spawn(SpawnIter(
            items.into_iter().map(ItemEntry::bundle),
        )))
    }

    fn on_shown(_: On<ScreenShown>, mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    fn on_hidden(_: On<ScreenHidden>, mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
        cursor.grab_mode = CursorGrabMode::Confined;
        cursor.visible = false;
    }

    fn on_respawn(
        _: On<Fire<Respawn>>,
        mut player: Single<(&mut Transform, &Player)>,
        point: Query<&GlobalTransform, With<MainSpawn>>,
        mut cmd: Commands,
    ) {
        if !player.1.main_spawn {
            return;
        }

        let Some(translation) = point.iter().next().map(|t| t.translation()) else {
            return;
        };

        player.0.translation = translation + Vec3::Y;

        cmd.set_state(GameState::InGame);
    }
}

#[derive(Component)]
struct ItemEntry;

impl ItemEntry {
    fn bundle(item: Item) -> impl Bundle {
        let name = (Text::new(item.name()), TextFont::from_font_size(30.0));

        let desc = (
            Text::new(item.description()),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
        );

        (
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(20),
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.3)),
            Outline::new(px(1), px(0), Color::WHITE),
            children![name, desc],
        )
    }
}
