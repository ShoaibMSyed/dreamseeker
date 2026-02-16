use avian3d::prelude::Position;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::prelude::Start;
use dreamseeker_util::{construct::Make, observers};

use crate::{
    GameState,
    input::ui::{Confirm, actions},
    player::{Player, item::Token},
    trigger::InitialSpawn,
};

use super::{Screen, ScreenCommandsExt, ScreenHidden, ScreenShown};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, EndScreen::update);
}

#[derive(Component, Reflect)]
#[require(Screen)]
pub struct EndScreen {
    color: Oklcha,
    timer: f32,
}

impl Default for EndScreen {
    fn default() -> Self {
        Self {
            color: Oklcha::new(0.8029, 0.084, 41.13, 1.0),
            timer: 3.0,
        }
    }
}

impl EndScreen {
    pub fn bundle() -> impl Bundle {
        let title = (
            Text::new("🎉🎉🎉 Winner! 🎉🎉🎉"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
        );

        let body = Make(Self::make_body);

        (
            EndScreen::default(),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.5)),
            actions(),
            observers![Self::on_shown, Self::on_hidden, Self::on_confirm],
            children![title, body],
        )
    }

    fn make_body(player: Single<&Player>, tokens: Query<&Token>) -> Result<impl Bundle + use<>> {
        let leftover = tokens.count();
        let tokens = player.dream_tokens as usize;
        let total = leftover + tokens;

        Ok((
            Text::new(format!(
                "You collected {tokens} / {total} tokens!\nPress Space / B to continue playing."
            )),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
        ))
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

    fn on_confirm(
        event: On<Start<Confirm>>,
        screen: Query<&EndScreen>,
        mut player: Single<&mut Position, With<Player>>,
        spawn: Query<&Transform, With<InitialSpawn>>,
        mut cmd: Commands,
    ) -> Result {
        if screen.get(event.context)?.timer > 0.0 {
            return Ok(());
        }

        if let Some(transform) = spawn.iter().next() {
            player.0 = transform.translation;
        }
        cmd.pop_screen();
        Ok(())
    }

    // fn on_new_game(
    //     event: On<Start<NewGame>>,
    //     screen: Query<&EndScreen>,
    //     checkpoints: Query<&mut Checkpoint>,
    //     mut player: Single<(&mut Position, &mut Player)>,
    //     // spawn: Query<&Transform, With<InitialSpawn>>,
    //     scene: Single<Entity, With<MainScene>>,
    //     mut cmd: Commands,
    // ) -> Result {
    //     if screen.get(event.context)?.timer > 0.0 {
    //         return Ok(());
    //     }

    //     cmd.pop_screen();

    //     for mut c in checkpoints {
    //         c.checked = true;
    //     }

    //     player.1.dream_tokens = 0;
    //     player.1.last_checkpoint = None;

    //     cmd.entity(*scene).despawn();
    //     cmd.spawn(MainScene::bundle());

    //     Ok(())
    // }

    fn update(q: Query<(&mut EndScreen, &mut BackgroundColor)>, time: Res<Time>) {
        for (mut screen, mut bg) in q {
            if screen.timer > 0.0 {
                screen.timer -= time.delta_secs();
            }
            screen.color.hue += time.delta_secs();
            screen.color.hue = screen.color.hue % 360.0;
            bg.0 = screen.color.into();
        }
    }
}
