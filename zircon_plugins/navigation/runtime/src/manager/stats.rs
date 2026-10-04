use zircon_runtime::core::framework::navigation::{
    NavigationRuntimeStats, NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
};
use zircon_runtime::scene::World;

pub(super) fn count_obstacles(world: &World) -> usize {
    world
        .node_records()
        .into_iter()
        .filter(|node| {
            world
                .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
                .is_some()
        })
        .count()
}

pub(super) fn count_navigation_components(world: &World) -> NavigationRuntimeStats {
    let mut stats = NavigationRuntimeStats::default();
    // Use fresh projections so editor/runtime dynamic component writes are counted immediately.
    for node in world.node_records() {
        if world
            .dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_agents += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_obstacles += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_off_mesh_links += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_off_mesh_bridges += 1;
        }
    }
    stats
}

#[cfg(test)]
#[path = "tests/stats.rs"]
mod tests;
