use bevy::prelude::*;
use bevy_enhanced_input::prelude::Start;
use dreamseeker_util::{construct::Make, observers};

use crate::{
    input::ui::{Confirm, actions},
    player::{Die, Player},
    trigger::Checkpoint,
};

use super::ScreenCommandsExt;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, TeleportScreen::update);
}

#[derive(Component, Reflect)]
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
            observers![Self::on_confirm],
        )
    }

    fn make(q: Query<(Entity, &Checkpoint)>) -> Result<impl Bundle + use<>> {
        let entries = q
            .iter()
            .filter(|(_, c)| c.checked)
            .map(|(e, c)| (e, c.id.clone()))
            .collect::<Vec<_>>();

        let selector = (Selector, Text::new("None"));

        let list = (Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },);

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
                    format!("None")
                } else {
                    format!("{}", screen.selected)
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
}

#[derive(Component)]
struct Selector;
