use zircon_runtime::core::framework::navigation::NAV_MESH_OBSTACLE_COMPONENT_TYPE;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub(super) fn nav_mesh_obstacle_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(
        NAV_MESH_OBSTACLE_COMPONENT_TYPE,
        PLUGIN_ID,
        "Nav Mesh Obstacle",
    )
    .with_property("shape", "navigation_obstacle_shape", true)
    .with_property("center", "vec3", true)
    .with_property("size", "vec3", true)
    .with_property("radius", "scalar", true)
    .with_property("height", "scalar", true)
    .with_property("avoidance_enabled", "bool", true)
    .with_property("carve", "bool", true)
    .with_property("move_threshold", "scalar", true)
    .with_property("time_to_stationary", "scalar", true)
    .with_property("carve_only_stationary", "bool", true)
}
