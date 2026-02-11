use std::{marker::PhantomData, panic::Location};

use bevy::{
    ecs::{
        bundle::DynamicBundle,
        error::{DefaultErrorHandler, ErrorContext},
        system::{RunSystemError, RunSystemOnce},
    },
    prelude::*,
};

#[derive(Resource, Clone, Default, Deref)]
pub struct Constructing(pub Option<Entity>);

pub struct MakeConstruct<C, I: SystemInput, B, M>(
    pub C,
    pub I::Inner<'static>,
    PhantomData<(B, M)>,
    Location<'static>,
);

#[track_caller]
#[allow(non_snake_case, unused)]
pub fn Make<C, B, M>(construct: C) -> MakeConstruct<C, (), B, M>
where
    C: Construct<(), B, M>,
    B: Bundle,
{
    MakeConstruct(construct, (), PhantomData, *Location::caller())
}

#[track_caller]
#[allow(non_snake_case, unused)]
pub fn MakeWith<C, I, B, M>(construct: C, input: I::Inner<'static>) -> MakeConstruct<C, I, B, M>
where
    C: Construct<I, B, M>,
    I: SystemInput,
    B: Bundle,
{
    MakeConstruct(construct, input, PhantomData, *Location::caller())
}

impl<C, I, B, M> DynamicBundle for MakeConstruct<C, I, B, M>
where
    C: Construct<I, B, M>,
    I: SystemInput,
    B: Bundle,
{
    type Effect = Self;

    unsafe fn get_components(
        _ptr: bevy::ptr::MovingPtr<'_, Self>,
        _func: &mut impl FnMut(bevy::ecs::component::StorageType, bevy::ptr::OwningPtr<'_>),
    ) {
    }

    unsafe fn apply_effect(
        ptr: bevy::ptr::MovingPtr<'_, std::mem::MaybeUninit<Self>>,
        entity: &mut EntityWorldMut,
    ) {
        let effect = unsafe { ptr.assume_init() };
        let this = effect.read();

        let entity_id = entity.id();

        let (result, tick) = entity.world_scope(move |world| {
            let tick = world.change_tick();

            world.init_resource::<Constructing>();

            let prev = world.resource::<Constructing>().clone();

            world.insert_resource(Constructing(Some(entity_id)));

            let ret = (this.0.construct(this.1, world), tick);

            world.insert_resource(prev);

            ret
        });

        let bundle = match result {
            Ok(bundle) => bundle,
            Err(e) => {
                error!("error running construct at {}", this.3);
                entity.world().resource::<DefaultErrorHandler>().0(
                    e,
                    ErrorContext::System {
                        name: "apply construct".into(),
                        last_run: tick,
                    },
                );
                return;
            }
        };

        entity.insert(bundle);
    }
}

unsafe impl<C, I, B, M> Bundle for MakeConstruct<C, I, B, M>
where
    C: Construct<I, B, M>,
    I: SystemInput,
    B: Bundle,
    MakeConstruct<C, I, B, M>: 'static + Send + Sync,
{
    fn component_ids(
        _components: &mut bevy::ecs::component::ComponentsRegistrator,
    ) -> impl Iterator<Item = bevy::ecs::component::ComponentId> + use<C, I, B, M> {
        std::iter::empty()
    }

    fn get_component_ids(_components: &bevy::ecs::component::Components) -> impl Iterator<Item = Option<bevy::ecs::component::ComponentId>> {
        std::iter::empty()
    }
}

pub trait Construct<I: SystemInput, B, M>: 'static + Send + Sync {
    fn construct(self, args: I::Inner<'static>, world: &mut World) -> Result<B>;
}

impl<C, I, B, M> Construct<I, B, M> for C
where
    C: IntoSystem<I, Result<B>, M>,
    I: SystemInput,
    B: Bundle,
    C: 'static + Send + Sync,
{
    fn construct(self, args: I::Inner<'static>, world: &mut World) -> Result<B> {
        let bundle = match world.run_system_once_with(self, args) {
            Ok(result) => result?,
            Err(RunSystemError::Failed(e)) => return Err(e),
            Err(RunSystemError::Skipped(e)) => return Err(e.into()),
        };
        Ok(bundle)
    }
}
