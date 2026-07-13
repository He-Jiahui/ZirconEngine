use zircon_runtime::core::framework::navigation::NAV_MESH_MODIFIER_COMPONENT_TYPE;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub(super) fn nav_mesh_modifier_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(
        NAV_MESH_MODIFIER_COMPONENT_TYPE,
        PLUGIN_ID,
        "Nav Mesh Modifier",
    )
    .with_property("mode", "navigation_modifier_mode", true)
    .with_property("affected_agents", "string_list", true)
    .with_property("apply_to_children", "bool", true)
    .with_property("override_area", "bool", true)
    .with_property("area", "navigation_area", true)
    .with_property("override_generate_links", "bool", true)
    .with_property("generate_links", "bool", true)
}
