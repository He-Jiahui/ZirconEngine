use super::*;
use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};

#[test]
fn navigation_editor_plugin_contributes_authoring_extensions() {
    let registration = plugin_registration();

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .capabilities
        .contains(&NAVIGATION_AUTHORING_CAPABILITY.to_string()));
    for view_id in [
        NAVIGATION_AUTHORING_VIEW_ID,
        NAVIGATION_AGENTS_VIEW_ID,
        NAVIGATION_BAKE_VIEW_ID,
        NAVIGATION_DEBUG_VIEW_ID,
    ] {
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == view_id));
    }
    assert!(registration
        .extensions
        .drawers()
        .iter()
        .any(|drawer| drawer.id() == NAVIGATION_DRAWER_ID));
    for template_id in [
        NAVIGATION_TEMPLATE_ID,
        NAVIGATION_AGENTS_TEMPLATE_ID,
        NAVIGATION_BAKE_TEMPLATE_ID,
        NAVIGATION_DEBUG_TEMPLATE_ID,
        NAVIGATION_ASSET_TEMPLATE_ID,
        NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID,
    ] {
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == template_id));
    }
    for component_type in [
        NAV_MESH_SURFACE_COMPONENT_TYPE,
        NAV_MESH_MODIFIER_COMPONENT_TYPE,
        NAV_MESH_AGENT_COMPONENT_TYPE,
        NAV_MESH_OBSTACLE_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    ] {
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| drawer.component_type() == component_type));
    }
    for operation in [
        "view.navigation.surfaces.open",
        "view.navigation.agents_areas.open",
        "view.navigation.bake.open",
        "view.navigation.debug_gizmos.open",
        NAVIGATION_BAKE_SCENE_OPERATION,
        NAVIGATION_BAKE_SURFACE_OPERATION,
        NAVIGATION_CLEAR_SURFACE_OPERATION,
        NAVIGATION_OPEN_SETTINGS_OPERATION,
        NAVIGATION_TOGGLE_GIZMOS_OPERATION,
        NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION,
        NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION,
    ] {
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|descriptor| descriptor.path().as_str() == operation));
    }
    assert!(registration
        .extensions
        .asset_editors()
        .iter()
        .any(|editor| editor.asset_kind() == "NavMesh"));
    assert!(registration
        .extensions
        .asset_editors()
        .iter()
        .any(|editor| editor.asset_kind() == "NavigationSettings"));

    for document in navigation_editor_documents() {
        assert!(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(document)
                .exists(),
            "missing navigation editor document {document}"
        );
    }
}

fn navigation_editor_documents() -> &'static [&'static str] {
    &[
        "surfaces.zui",
        "agents_areas.zui",
        "bake.zui",
        "debug_gizmos.zui",
        "navmesh_asset.zui",
        "navigation_settings_asset.zui",
        "navmesh_surface.drawer.zui",
        "navmesh_modifier.drawer.zui",
        "navmesh_agent.drawer.zui",
        "navmesh_obstacle.drawer.zui",
        "navmesh_offmesh_link.drawer.zui",
    ]
}
