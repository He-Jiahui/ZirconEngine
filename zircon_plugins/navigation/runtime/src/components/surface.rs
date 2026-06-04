use zircon_runtime::core::framework::navigation::NAV_MESH_SURFACE_COMPONENT_TYPE;
use zircon_runtime::plugin::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub(super) fn nav_mesh_surface_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(
        NAV_MESH_SURFACE_COMPONENT_TYPE,
        PLUGIN_ID,
        "Nav Mesh Surface",
    )
    .with_property("enabled", "bool", true)
    .with_property("agent_type", "string", true)
    .with_property("collect_mode", "navigation_collect_mode", true)
    .with_property("volume_center", "vec3", true)
    .with_property("volume_size", "vec3", true)
    .with_property("use_geometry", "navigation_use_geometry", true)
    .with_property("include_layers", "string_list", true)
    .with_property("default_area", "navigation_area", true)
    .with_property("generate_links", "bool", true)
    .with_property("override_voxel_size", "optional_scalar", true)
    .with_property("override_tile_size", "optional_unsigned", true)
    .with_property("min_region_area", "scalar", true)
    .with_property("build_height_mesh", "bool", true)
    .with_property("output_asset", "resource", true)
}
