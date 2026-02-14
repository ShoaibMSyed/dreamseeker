use bevy::prelude::*;

use super::{Screen, ScreenCommandsExt};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, Black::update)
        .add_observer(Black::on_end);
}

#[derive(Event)]
pub struct EndTransition;

#[derive(Component)]
#[require(Screen)]
pub struct TransScreen(Option<Box<dyn FnMut(&mut World) + Send + Sync>>);

impl TransScreen {
    pub fn bundle<M: Message + Clone>(ready_message: M) -> impl Bundle {
        let f = Box::new(move |world: &mut World| {
            world.write_message(ready_message.clone());
        });

        (
            Self(Some(f)),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Children::spawn_one(Black::default()),
        )
    }
}

#[derive(Component, Default)]
#[require(
    Node {
        width: percent(100),
        height: percent(100),
        position_type: PositionType::Absolute,
        left: percent(-100),
        ..default()
    },
    BackgroundColor(Color::BLACK),
)]
enum Black {
    #[default]
    Closing,
    Waiting,
    Opening,
}

impl Black {
    fn update(
        black: Single<(&mut Node, &mut Black, &ChildOf), With<Black>>,
        mut trans: Query<&mut TransScreen>,
        time: Res<Time>,
        mut cmd: Commands,
    ) {
        let (mut node, mut black, parent) = black.into_inner();

        let Val::Percent(p) = &mut node.left else {
            return;
        };

        match &mut *black {
            Black::Closing => {
                *p += 350.0 * time.delta_secs();
                if *p >= 0.0 {
                    *p = 0.0;
                    if let Some(c) = trans.get_mut(parent.0).ok().and_then(|mut t| t.0.take()) {
                        cmd.queue(move |world: &mut World| {
                            let c = c;
                            c.apply(world)
                        });
                    }
                    *black = Black::Waiting;
                }
            }
            Black::Waiting => {}
            Black::Opening => {
                *p += 350.0 * time.delta_secs();
                if *p >= 100.0 {
                    *p = 100.0;
                    cmd.pop_screen();
                }
            }
        }
    }

    fn on_end(_: On<EndTransition>, black: Query<&mut Black>) {
        for mut black in black {
            *black = Black::Opening;
        }
    }
}
