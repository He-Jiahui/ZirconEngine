mod agent;
mod modifier;
mod obstacle;
mod off_mesh_bridge;
mod off_mesh_link;
mod surface;

use zircon_runtime::plugin::ComponentTypeDescriptor;

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
