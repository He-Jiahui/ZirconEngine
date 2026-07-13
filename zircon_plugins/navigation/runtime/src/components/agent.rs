use zircon_runtime::core::framework::navigation::{
    NAV_DESIRED_VELOCITY_COMPONENT_TYPE, NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub(super) fn nav_mesh_agent_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(NAV_MESH_AGENT_COMPONENT_TYPE, PLUGIN_ID, "Nav Mesh Agent")
        .with_property("nav_mesh", "optional_navigation_mesh_handle", true)
        .with_property("agent_type", "string", true)
        .with_property("radius", "scalar", true)
        .with_property("height", "scalar", true)
        .with_property("base_offset", "scalar", true)
        .with_property("speed", "scalar", true)
        .with_property("angular_speed", "scalar", true)
        .with_property("acceleration", "scalar", true)
        .with_property("stopping_distance", "scalar", true)
        .with_property("auto_braking", "bool", true)
        .with_property("avoidance_quality", "navigation_avoidance_quality", true)
        .with_property("priority", "unsigned", true)
        .with_property("area_mask", "unsigned", true)
        .with_property("auto_repath", "bool", true)
        .with_property("auto_traverse_links", "bool", true)
        .with_property("update_position", "bool", true)
        .with_property("update_rotation", "bool", true)
        .with_property("writeback_mode", "navigation_agent_writeback_mode", true)
        .with_property("destination", "optional_vec3", true)
}

pub(super) fn nav_desired_velocity_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(
        NAV_DESIRED_VELOCITY_COMPONENT_TYPE,
        PLUGIN_ID,
        "Navigation Desired Velocity",
    )
    .with_property("linear", "vec3", false)
}
