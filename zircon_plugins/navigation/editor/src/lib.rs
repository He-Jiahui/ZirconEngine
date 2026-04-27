use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = zircon_plugin_navigation_runtime::PLUGIN_ID;
pub const NAVIGATION_AUTHORING_VIEW_ID: &str = "navigation.authoring";
pub const NAVIGATION_DRAWER_ID: &str = "navigation.drawer";
pub const NAVIGATION_TEMPLATE_ID: &str = "navigation.authoring";

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
                template_document: "plugins://navigation/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    NAVIGATION_AUTHORING_VIEW_ID,
                    "Navigation",
                    "World",
                    "Plugins/Navigation",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Navigation",
        "zircon_plugin_navigation_editor",
    )
    .with_capability("editor.extension.navigation_authoring")
}

pub fn editor_plugin() -> NavigationEditorPlugin {
    NavigationEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
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
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == NAVIGATION_AUTHORING_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == NAVIGATION_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == NAVIGATION_TEMPLATE_ID));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.navigation.authoring.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.navigation.authoring.Open"));
    }
}
