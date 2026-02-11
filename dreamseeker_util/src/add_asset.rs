use bevy::{ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*};

#[derive(Component)]
#[component(on_insert)]
struct AddAssetInternal<A: Asset, C: Component, F: FnOnce(Handle<A>) -> C + Send + Sync + 'static>(Option<(A, F)>);

impl<A: Asset, C: Component, F: FnOnce(Handle<A>) -> C + Send + Sync + 'static> AddAssetInternal<A, C, F> {
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let (asset, constructor) = world.get_mut::<Self>(ctx.entity).unwrap().0.take().unwrap();

        let handle = world.resource_mut::<Assets<A>>().add(asset);
        let component = constructor(handle);
        world.commands().entity(ctx.entity).insert(component);
    }
}

#[allow(non_snake_case)]
pub fn AddAsset<A: Asset, C: Component, F: FnOnce(Handle<A>) -> C + Send + Sync + 'static>(asset: A, constructor: F) -> impl Bundle {
    AddAssetInternal(Some((asset, constructor)))
}

#[allow(non_snake_case)]
pub fn AddMesh(mesh: impl Into<Mesh>) -> impl Bundle {
    AddAsset::<Mesh, Mesh3d, _>(mesh.into(), Mesh3d)
}

#[allow(non_snake_case)]
pub fn AddMaterial(mat: impl Into<StandardMaterial>) -> impl Bundle {
    AddAsset::<StandardMaterial, MeshMaterial3d<StandardMaterial>, _>(mat.into(), MeshMaterial3d)
}