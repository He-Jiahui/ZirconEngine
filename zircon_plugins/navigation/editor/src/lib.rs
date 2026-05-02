use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};

pub const PLUGIN_ID: &str = zircon_plugin_navigation_runtime::PLUGIN_ID;
pub const NAVIGATION_AUTHORING_VIEW_ID: &str = "navigation.surfaces";
pub const NAVIGATION_AGENTS_VIEW_ID: &str = "navigation.agents_areas";
pub const NAVIGATION_BAKE_VIEW_ID: &str = "navigation.bake";
pub const NAVIGATION_DEBUG_VIEW_ID: &str = "navigation.debug_gizmos";
pub const NAVIGATION_ASSET_VIEW_ID: &str = "navigation.navmesh_asset";
pub const NAVIGATION_SETTINGS_ASSET_VIEW_ID: &str = "navigation.settings_asset";
pub const NAVIGATION_DRAWER_ID: &str = "navigation.drawer";
pub const NAVIGATION_TEMPLATE_ID: &str = "navigation.surfaces";
pub const NAVIGATION_AGENTS_TEMPLATE_ID: &str = "navigation.agents_areas";
pub const NAVIGATION_BAKE_TEMPLATE_ID: &str = "navigation.bake";
pub const NAVIGATION_DEBUG_TEMPLATE_ID: &str = "navigation.debug_gizmos";
pub const NAVIGATION_ASSET_TEMPLATE_ID: &str = "navigation.navmesh_asset";
pub const NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID: &str = "navigation.settings_asset";
pub const NAV_MESH_SURFACE_DRAWER_ID: &str = "navigation.Component.NavMeshSurface.drawer";
pub const NAV_MESH_MODIFIER_DRAWER_ID: &str = "navigation.Component.NavMeshModifier.drawer";
pub const NAV_MESH_AGENT_DRAWER_ID: &str = "navigation.Component.NavMeshAgent.drawer";
pub const NAV_MESH_OBSTACLE_DRAWER_ID: &str = "navigation.Component.NavMeshObstacle.drawer";
pub const NAV_MESH_OFF_MESH_LINK_DRAWER_ID: &str = "navigation.Component.NavMeshOffMeshLink.drawer";
pub const NAVIGATION_BAKE_SCENE_OPERATION: &str = "Navigation.Bake.Scene";
pub const NAVIGATION_BAKE_SURFACE_OPERATION: &str = "Navigation.Bake.Surface";
pub const NAVIGATION_CLEAR_SURFACE_OPERATION: &str = "Navigation.Bake.ClearSurface";
pub const NAVIGATION_OPEN_SETTINGS_OPERATION: &str = "Navigation.Settings.Open";
pub const NAVIGATION_TOGGLE_GIZMOS_OPERATION: &str = "Navigation.Debug.ToggleGizmos";
pub const NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION: &str = "Navigation.Asset.OpenNavMesh";
pub const NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION: &str = "Navigation.Asset.OpenSettings";

#[derive(Clone, Debug)]
pub struct NavigationEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl NavigationEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl Default for NavigationEditorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for NavigationEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: NAVIGATION_DRAWER_ID,
                drawer_display_name: "Navigation Tools",
                template_id: NAVIGATION_TEMPLATE_ID,
                template_document: "plugins://navigation/editor/surfaces.ui.toml",
                surfaces: &[
                    EditorAuthoringSurface::new(
                        NAVIGATION_AUTHORING_VIEW_ID,
                        "Navigation Surfaces",
                        "World",
                        "Plugins/Navigation/Surfaces",
                    ),
                    EditorAuthoringSurface::new(
                        NAVIGATION_AGENTS_VIEW_ID,
                        "Navigation Agents & Areas",
                        "World",
                        "Plugins/Navigation/Agents & Areas",
                    ),
                    EditorAuthoringSurface::new(
                        NAVIGATION_BAKE_VIEW_ID,
                        "Navigation Bake",
                        "World",
                        "Plugins/Navigation/Bake",
                    ),
                    EditorAuthoringSurface::new(
                        NAVIGATION_DEBUG_VIEW_ID,
                        "Navigation Debug",
                        "World",
                        "Plugins/Navigation/Debug",
                    ),
                ],
            },
        )?;
        register_navigation_templates(registry)?;
        register_navigation_component_drawers(registry)?;
        register_navigation_operations(registry)?;
        register_navigation_asset_editor(registry)
    }
}

fn register_navigation_templates(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_extension::EditorUiTemplateDescriptor;

    for (id, document) in [
        (
            NAVIGATION_AGENTS_TEMPLATE_ID,
            "plugins://navigation/editor/agents_areas.ui.toml",
        ),
        (
            NAVIGATION_BAKE_TEMPLATE_ID,
            "plugins://navigation/editor/bake.ui.toml",
        ),
        (
            NAVIGATION_DEBUG_TEMPLATE_ID,
            "plugins://navigation/editor/debug_gizmos.ui.toml",
        ),
        (
            NAVIGATION_ASSET_TEMPLATE_ID,
            "plugins://navigation/editor/navmesh_asset.ui.toml",
        ),
        (
            NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID,
            "plugins://navigation/editor/navigation_settings_asset.ui.toml",
        ),
    ] {
        registry.register_ui_template(EditorUiTemplateDescriptor::new(id, document))?;
    }
    Ok(())
}

fn register_navigation_component_drawers(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_extension::ComponentDrawerDescriptor;

    for (component_type, document, controller) in [
        (
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_surface.drawer.ui.toml",
            NAV_MESH_SURFACE_DRAWER_ID,
        ),
        (
            NAV_MESH_MODIFIER_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_modifier.drawer.ui.toml",
            NAV_MESH_MODIFIER_DRAWER_ID,
        ),
        (
            NAV_MESH_AGENT_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_agent.drawer.ui.toml",
            NAV_MESH_AGENT_DRAWER_ID,
        ),
        (
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_obstacle.drawer.ui.toml",
            NAV_MESH_OBSTACLE_DRAWER_ID,
        ),
        (
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
            "plugins://navigation/editor/navmesh_offmesh_link.drawer.ui.toml",
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

fn register_navigation_operations(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_event::{EditorEvent, MenuAction, ViewDescriptorId};
    use zircon_editor::core::editor_extension::EditorMenuItemDescriptor;
    use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    for (path, display_name, menu_path) in [
        (
            NAVIGATION_BAKE_SCENE_OPERATION,
            "Bake Navigation Scene",
            "Plugins/Navigation/Bake Scene",
        ),
        (
            NAVIGATION_BAKE_SURFACE_OPERATION,
            "Bake Selected NavMesh Surface",
            "Plugins/Navigation/Bake Selected Surface",
        ),
        (
            NAVIGATION_CLEAR_SURFACE_OPERATION,
            "Clear NavMesh Surface Bake",
            "Plugins/Navigation/Clear Surface Bake",
        ),
        (
            NAVIGATION_OPEN_SETTINGS_OPERATION,
            "Open Navigation Settings",
            "Plugins/Navigation/Settings",
        ),
        (
            NAVIGATION_TOGGLE_GIZMOS_OPERATION,
            "Toggle Navigation Gizmos",
            "Plugins/Navigation/Toggle Gizmos",
        ),
    ] {
        let operation_path = EditorOperationPath::parse(path).map_err(
            zircon_editor::core::editor_extension::EditorExtensionRegistryError::Operation,
        )?;
        let mut descriptor = EditorOperationDescriptor::new(operation_path.clone(), display_name)
            .with_menu_path(menu_path)
            .with_callable_from_remote(false);
        descriptor = match path {
            NAVIGATION_OPEN_SETTINGS_OPERATION => {
                descriptor.with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                    ViewDescriptorId::new(NAVIGATION_AGENTS_VIEW_ID),
                )))
            }
            NAVIGATION_TOGGLE_GIZMOS_OPERATION => {
                descriptor.with_event(EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                    ViewDescriptorId::new(NAVIGATION_DEBUG_VIEW_ID),
                )))
            }
            _ => descriptor,
        };
        registry.register_operation(
            descriptor.with_required_capabilities(["editor.extension.navigation_authoring"]),
        )?;
        registry.register_menu_item(EditorMenuItemDescriptor::new(menu_path, operation_path))?;
    }
    Ok(())
}

fn register_navigation_asset_editor(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_extension::AssetEditorDescriptor;
    use zircon_editor::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let operation_path = EditorOperationPath::parse(NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION)
        .map_err(zircon_editor::core::editor_extension::EditorExtensionRegistryError::Operation)?;
    registry.register_operation(
        EditorOperationDescriptor::new(operation_path.clone(), "Open NavMesh Asset")
            .with_callable_from_remote(false),
    )?;
    registry.register_asset_editor(AssetEditorDescriptor::new(
        "NavMesh",
        NAVIGATION_ASSET_VIEW_ID,
        "NavMesh Asset",
        operation_path,
    ))?;

    let operation_path = EditorOperationPath::parse(NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION)
        .map_err(zircon_editor::core::editor_extension::EditorExtensionRegistryError::Operation)?;
    registry.register_operation(
        EditorOperationDescriptor::new(operation_path.clone(), "Open Navigation Settings Asset")
            .with_callable_from_remote(false),
    )?;
    registry.register_asset_editor(AssetEditorDescriptor::new(
        "NavigationSettings",
        NAVIGATION_SETTINGS_ASSET_VIEW_ID,
        "Navigation Settings Asset",
        operation_path,
    ))
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Navigation",
        "zircon_plugin_navigation_editor",
    )
    .with_capability("editor.extension.navigation_authoring")
    .with_capability("editor.extension.navigation_gizmos")
}

pub fn editor_plugin() -> NavigationEditorPlugin {
    NavigationEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_navigation_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_navigation_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_editor_plugin_contributes_authoring_extensions() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&"editor.extension.navigation_authoring".to_string()));
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
            "View.navigation.surfaces.Open",
            "View.navigation.agents_areas.Open",
            "View.navigation.bake.Open",
            "View.navigation.debug_gizmos.Open",
            NAVIGATION_BAKE_SCENE_OPERATION,
            NAVIGATION_CLEAR_SURFACE_OPERATION,
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
    }
}
