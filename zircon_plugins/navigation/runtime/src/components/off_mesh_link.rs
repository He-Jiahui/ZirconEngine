use zircon_runtime::core::framework::navigation::NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub(super) fn nav_mesh_off_mesh_link_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(
        NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        PLUGIN_ID,
        "Nav Mesh Off Mesh Link",
    )
    .with_property("start_entity", "entity", true)
    .with_property("end_entity", "entity", true)
    .with_property("start_local_point", "vec3", true)
    .with_property("end_local_point", "vec3", true)
    .with_property("width", "scalar", true)
    .with_property("bidirectional", "bool", true)
    .with_property("activated", "bool", true)
    .with_property("auto_update_positions", "bool", true)
    .with_property("cost_override", "optional_scalar", true)
    .with_property("area_type", "navigation_area", true)
    .with_property("agent_type", "string", true)
    .with_property("traversal_mode", "navigation_link_traversal_mode", true)
    .with_property("motion", "navigation_link_motion", true)
    .with_property("arc_height", "scalar", true)
}
