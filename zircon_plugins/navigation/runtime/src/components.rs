use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::plugin::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub fn navigation_component_descriptors() -> Vec<ComponentTypeDescriptor> {
    vec![
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
        .with_property("output_asset", "resource", true),
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
        .with_property("generate_links", "bool", true),
        ComponentTypeDescriptor::new(NAV_MESH_AGENT_COMPONENT_TYPE, PLUGIN_ID, "Nav Mesh Agent")
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
            .with_property("destination", "optional_vec3", true),
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
        .with_property("carve_only_stationary", "bool", true),
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
        .with_property("traversal_mode", "navigation_link_traversal_mode", true),
    ]
}
