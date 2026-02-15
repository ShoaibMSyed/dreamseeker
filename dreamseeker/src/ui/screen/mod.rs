use bevy::{ecs::entity_disabling::Disabled, prelude::*};

pub mod hud;
pub mod info;
pub mod item;
pub mod pause;
pub mod teleport;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        self::hud::plugin,
        self::info::plugin,
        self::item::plugin,
        self::pause::plugin,
        self::teleport::plugin,
    ))
    .init_resource::<ScreenStack>();
}

#[derive(Resource, Reflect, Clone, Default)]
pub struct ScreenStack(Vec<Entity>);

impl ScreenStack {
    pub fn current(&self) -> Option<Entity> {
        self.0.last().copied()
    }
}

#[derive(Component, Reflect, Default)]
pub struct Screen {
    pub priority: i32,
}

pub struct PushScreen<B>(pub B);

impl<B: Bundle> Command for PushScreen<B> {
    fn apply(self, world: &mut World) -> () {
        if let Some(&e) = world.resource::<ScreenStack>().0.last() {
            world.entity_mut(e).insert_recursive::<Children>(Disabled);
        }

        let entity = world.spawn(self.0).id();
        world.resource_mut::<ScreenStack>().0.push(entity);
        world.trigger(ScreenShown(entity));
    }
}

pub struct PopScreen;

impl Command for PopScreen {
    fn apply(self, world: &mut World) -> () {
        let Some(&entity) = world.resource::<ScreenStack>().0.last() else {
            return;
        };
        world.trigger(ScreenHidden(entity));
        world.resource_mut::<ScreenStack>().0.pop();
        world.despawn(entity);
        if let Some(&e) = world.resource::<ScreenStack>().0.last() {
            world.entity_mut(e).remove_recursive::<Children, Disabled>();
            world.trigger(ScreenShown(e));
        }
    }
}

pub trait ScreenCommandsExt {
    fn push_screen(&mut self, bundle: impl Bundle);
    #[allow(dead_code)]
    fn set_screen(&mut self, bundle: impl Bundle) {
        self.pop_screen();
        self.push_screen(bundle);
    }
    fn pop_screen(&mut self);
}

impl ScreenCommandsExt for Commands<'_, '_> {
    fn push_screen(&mut self, bundle: impl Bundle) {
        self.queue(PushScreen(bundle));
    }

    fn pop_screen(&mut self) {
        self.queue(PopScreen);
    }
}

#[derive(EntityEvent, Clone)]
pub struct ScreenHidden(pub Entity);

#[derive(EntityEvent, Clone)]
pub struct ScreenShown(pub Entity);
