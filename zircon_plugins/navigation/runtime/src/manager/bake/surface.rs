use crate::component_json::parse_component;
use zircon_runtime::core::framework::navigation::{
    NavMeshSurfaceDescriptor, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::scene::World;

pub(super) fn collect_surfaces(world: &World) -> Vec<(u64, NavMeshSurfaceDescriptor)> {
    // Navigation authoring scans must see direct world mutations before PostUpdate refreshes nodes().
    world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_SURFACE_COMPONENT_TYPE)?;
            let surface = parse_component::<NavMeshSurfaceDescriptor>(value);
            surface.enabled.then_some((node.id, surface))
        })
        .collect()
}

pub(super) fn select_bake_surface(
    surfaces: &[(u64, NavMeshSurfaceDescriptor)],
    requested_entity: Option<u64>,
) -> Option<(u64, NavMeshSurfaceDescriptor)> {
    requested_entity
        .and_then(|entity| surfaces.iter().find(|(id, _)| *id == entity).cloned())
        .or_else(|| surfaces.first().cloned())
}
