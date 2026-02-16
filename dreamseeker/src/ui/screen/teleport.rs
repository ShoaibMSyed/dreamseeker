use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::Start;
use dreamseeker_util::{construct::Make, observers};

use crate::{
    GameState,
    input::ui::{Confirm, Move, actions},
    player::{Die, Player},
    trigger::Checkpoint,
};

use super::{Screen, ScreenCommandsExt, ScreenHidden, ScreenShown};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, TeleportScreen::update);
}

#[derive(Component, Reflect)]
#[require(Screen)]
pub struct TeleportScreen {
    entries: Vec<(Entity, String)>,
    selected: usize,
}

impl TeleportScreen {
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.5)),
            actions(),
            Make(Self::make),
            observers![
                Self::on_confirm,
                Self::on_shown,
                Self::on_hidden,
                Self::on_move
            ],
        )
    }

    fn make(q: Query<(Entity, &Checkpoint)>) -> Result<impl Bundle + use<>> {
        let entries = q
            .iter()
            .filter(|(_, c)| c.checked)
            .map(|(e, c)| (e, c.id.clone()))
            .collect::<Vec<_>>();

        let selector = (
            Selector,
            Text::new("None"),
            Node {
                padding: UiRect::all(px(10)),
                ..default()
            },
            Outline::new(px(1), px(0), Color::WHITE),
        );

        let list = (
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            Children::spawn(SpawnIter(entries.clone().into_iter().map(|(_, name)| {
                (
                    Text::new(name),
                    Outline::new(px(1), px(0), Color::WHITE),
                    Node {
                        padding: UiRect::all(px(5)),
                        ..default()
                    },
                )
            }))),
        );

        let info = (Text::new(
            "Move left and right to select a checkpoint\nUse Space / A to teleport",
        ),);

        let screen = TeleportScreen::new(entries);

        Ok((screen, children![selector, list, info,]))
    }

    fn new(entries: Vec<(Entity, String)>) -> Self {
        Self {
            entries,
            selected: 0,
        }
    }

    fn update(
        q: Query<(Entity, &TeleportScreen), Changed<TeleportScreen>>,
        q_children: Query<&Children>,
        mut selector: Query<&mut Text, With<Selector>>,
    ) -> Result {
        for (e, screen) in q {
            for desc in q_children.iter_descendants(e) {
                let Ok(mut text) = selector.get_mut(desc) else {
                    continue;
                };

                text.0 = if screen.entries.len() == 0 {
                    format!("Selected: None")
                } else {
                    format!("Selected: {}", screen.entries[screen.selected].1)
                };
            }
        }

        Ok(())
    }

    fn on_confirm(
        event: On<Start<Confirm>>,
        screen: Query<&TeleportScreen>,
        mut player: Single<(Entity, &mut Player)>,
        mut cmd: Commands,
    ) -> Result {
        let screen = screen.get(event.context)?;
        let Some(&(entry, _)) = screen.entries.get(screen.selected) else {
            return Ok(());
        };

        player.1.last_checkpoint = Some(entry);

        cmd.pop_screen();
        cmd.trigger(Die(player.0));
        Ok(())
    }

    fn on_move(event: On<Start<Move>>, mut screen: Query<&mut TeleportScreen>) -> Result {
        let mut screen = screen.get_mut(event.context)?;

        if screen.entries.is_empty() {
            return Ok(());
        }

        let next = event.value.x > 0.0;

        if next {
            screen.selected += 1;
            if screen.selected >= screen.entries.len() {
                screen.selected = 0;
            }
        } else {
            if screen.selected == 0 {
                screen.selected = screen.entries.len() - 1;
            } else {
                screen.selected -= 1;
            }
        }

        Ok(())
    }

    fn on_shown(
        _: On<ScreenShown>,
        mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
        mut state: ResMut<NextState<GameState>>,
    ) {
        state.set(GameState::Paused);
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    fn on_hidden(
        _: On<ScreenHidden>,
        mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
        mut state: ResMut<NextState<GameState>>,
    ) {
        state.set(GameState::InGame);
        cursor.grab_mode = CursorGrabMode::Confined;
        cursor.visible = false;
    }
}

#[derive(Component)]
struct Selector;
