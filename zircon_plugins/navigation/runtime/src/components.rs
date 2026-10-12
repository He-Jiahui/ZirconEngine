#[path = "components/agent.rs"]
mod agent;
#[path = "components/modifier.rs"]
mod modifier;
#[path = "components/obstacle.rs"]
mod obstacle;
#[path = "components/off_mesh_bridge.rs"]
mod off_mesh_bridge;
#[path = "components/off_mesh_link.rs"]
mod off_mesh_link;
#[path = "components/surface.rs"]
mod surface;

use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

pub fn navigation_component_descriptors() -> Vec<ComponentTypeDescriptor> {
    vec![
        surface::nav_mesh_surface_descriptor(),
        modifier::nav_mesh_modifier_descriptor(),
        agent::nav_mesh_agent_descriptor(),
        obstacle::nav_mesh_obstacle_descriptor(),
        off_mesh_link::nav_mesh_off_mesh_link_descriptor(),
        off_mesh_bridge::nav_mesh_off_mesh_bridge_descriptor(),
        agent::nav_desired_velocity_descriptor(),
    ]
}
