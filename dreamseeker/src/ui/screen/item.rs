use bevy::prelude::*;
use bevy_enhanced_input::prelude::Start;
use dreamseeker_util::observers;

use crate::{
    input::ui::{Confirm, actions},
    player::item::Item,
};

use super::{Screen, ScreenCommandsExt};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, ItemDescriptionScreen::update);
}

pub fn item_description(item: Item) -> impl Bundle {
    let name = (
        Text::new(item.name()),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
    );

    let description = (
        Text::new(item.description()),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
    );

    let exit = (
        Text::new("Press Space / B to exit"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
    );

    let column = (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(percent(10)),
            width: percent(50),
            ..default()
        },
        children![name, description, exit],
    );

    (
        ItemDescriptionScreen { cooldown: 1.0 },
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.5)),
        actions(),
        observers![ItemDescriptionScreen::on_confirm],
        children![column],
    )
}

#[derive(Component)]
#[require(Screen)]
pub struct ItemDescriptionScreen {
    cooldown: f32,
}

impl ItemDescriptionScreen {
    fn on_confirm(
        event: On<Start<Confirm>>,
        screen: Query<&ItemDescriptionScreen>,
        mut cmd: Commands,
    ) -> Result {
        let screen = screen.get(event.context)?;
        if screen.cooldown <= 0.0 {
            cmd.pop_screen();
        }
        Ok(())
    }

    fn update(screen: Query<&mut ItemDescriptionScreen>, time: Res<Time>) {
        for mut screen in screen {
            screen.cooldown = (screen.cooldown - time.delta_secs()).max(0.0);
        }
    }
}
