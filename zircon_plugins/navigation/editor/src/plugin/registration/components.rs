use zircon_editor::core::editor_extension::{
    ComponentDrawerDescriptor, EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};

use crate::extension_ids::{
    NAV_MESH_AGENT_DRAWER_ID, NAV_MESH_MODIFIER_DRAWER_ID, NAV_MESH_OBSTACLE_DRAWER_ID,
    NAV_MESH_OFF_MESH_LINK_DRAWER_ID, NAV_MESH_SURFACE_DRAWER_ID,
};

pub(super) fn register(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    for (component_type, document, controller) in [
        (
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_surface.drawer.zui",
            NAV_MESH_SURFACE_DRAWER_ID,
        ),
        (
            NAV_MESH_MODIFIER_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_modifier.drawer.zui",
            NAV_MESH_MODIFIER_DRAWER_ID,
        ),
        (
            NAV_MESH_AGENT_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_agent.drawer.zui",
            NAV_MESH_AGENT_DRAWER_ID,
        ),
        (
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_obstacle.drawer.zui",
            NAV_MESH_OBSTACLE_DRAWER_ID,
        ),
        (
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_offmesh_link.drawer.zui",
            NAV_MESH_OFF_MESH_LINK_DRAWER_ID,
        ),
    ] {
        registry.register_component_drawer(ComponentDrawerDescriptor::new(
            component_type,
            document,
            controller,
        ))?;
    }
    Ok(())
}
