use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = zircon_plugin_physics_runtime::PLUGIN_ID;
pub const PHYSICS_AUTHORING_VIEW_ID: &str = "physics.authoring";
pub const PHYSICS_DRAWER_ID: &str = "physics.drawer";
pub const PHYSICS_TEMPLATE_ID: &str = "physics.authoring";

#[derive(Clone, Debug)]
pub struct PhysicsEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl PhysicsEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for PhysicsEditorPlugin {
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
                drawer_id: PHYSICS_DRAWER_ID,
                drawer_display_name: "Physics Tools",
                template_id: PHYSICS_TEMPLATE_ID,
                template_document: "plugins://physics/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    PHYSICS_AUTHORING_VIEW_ID,
                    "Physics",
                    "World",
                    "Plugins/Physics",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Physics", "zircon_plugin_physics_editor")
        .with_capability("editor.extension.physics_authoring")
}

pub fn editor_plugin() -> PhysicsEditorPlugin {
    PhysicsEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_physics_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_physics_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_editor_plugin_contributes_authoring_extensions() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&"editor.extension.physics_authoring".to_string()));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == PHYSICS_AUTHORING_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == PHYSICS_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == PHYSICS_TEMPLATE_ID));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.physics.authoring.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.physics.authoring.Open"));
    }
}
