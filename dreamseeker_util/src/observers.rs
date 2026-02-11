use std::sync::Arc;

use bevy::{
    asset::uuid::Uuid,
    ecs::{lifecycle::HookContext, system::IntoObserverSystem, world::DeferredWorld},
    platform::collections::HashMap,
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ObserverMap>();
}

type ObserverFactory = Arc<dyn Fn() -> Observer + Send + Sync + 'static>;

#[derive(Resource, Default)]
pub struct ObserverMap(HashMap<Uuid, ObserverFactory>);

#[derive(Component, Reflect, Clone, Default)]
#[component(on_add)]
pub struct ObserverSpawner {
    spawn: Vec<Uuid>,
    #[reflect(ignore)]
    add: HashMap<Uuid, ObserverFactory>,
}

impl ObserverSpawner {
    pub fn with<E: Event, B: Bundle, M, I: IntoObserverSystem<E, B, M> + Clone + Send + Sync>(
        mut self,
        system: I,
    ) -> Self {
        let uuid = Uuid::new_v4();
        self.spawn.push(uuid);
        self.add
            .insert(uuid, Arc::new(move || Observer::new(system.clone())));
        self
    }

    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let mut spawner = world.get_mut::<Self>(ctx.entity).unwrap();
        let add = std::mem::take(&mut spawner.add);

        let to_spawn = spawner.spawn.clone();

        let mut map = world.resource_mut::<ObserverMap>();
        for (id, factory) in add {
            if !map.0.contains_key(&id) {
                map.0.insert(id, factory);
            }
        }

        let observers: Vec<Observer> = to_spawn
            .into_iter()
            .map(|id| map.0.get(&id).expect("observer not found")().with_entity(ctx.entity))
            .collect();

        world.commands().spawn_batch(observers);
    }
}

#[macro_export]
macro_rules! observers {
    [$($observer:expr),* $(,)?] => {
        $crate::observers::ObserverSpawner::default()
            $(.with($observer))*
    }
}
