use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = "runtime_diagnostics";
pub const RUNTIME_DIAGNOSTICS_VIEW_ID: &str = "editor.runtime_diagnostics";
pub const RUNTIME_DIAGNOSTICS_DRAWER_ID: &str = "runtime_diagnostics.drawer";
pub const RUNTIME_DIAGNOSTICS_TEMPLATE_ID: &str = "runtime_diagnostics.authoring";

#[derive(Clone, Debug)]
pub struct RuntimeDiagnosticsEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RuntimeDiagnosticsEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for RuntimeDiagnosticsEditorPlugin {
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
                drawer_id: RUNTIME_DIAGNOSTICS_DRAWER_ID,
                drawer_display_name: "Runtime Diagnostics Tools",
                template_id: RUNTIME_DIAGNOSTICS_TEMPLATE_ID,
                template_document: "pane.runtime_diagnostics.body",
                surfaces: &[EditorAuthoringSurface::new(
                    RUNTIME_DIAGNOSTICS_VIEW_ID,
                    "Runtime Diagnostics",
                    "Diagnostics",
                    "Plugins/Runtime Diagnostics",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Runtime Diagnostics",
        "zircon_plugin_runtime_diagnostics_editor",
    )
    .with_capability(zircon_editor::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS)
}

pub fn editor_plugin() -> RuntimeDiagnosticsEditorPlugin {
    RuntimeDiagnosticsEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_package_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        base_package_manifest(),
    )
}

fn base_package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::PluginPackageManifest::new(PLUGIN_ID, "Runtime Diagnostics")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_diagnostics_plugin_contributes_view_and_capability() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&zircon_editor::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS.to_string()));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == RUNTIME_DIAGNOSTICS_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == RUNTIME_DIAGNOSTICS_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == RUNTIME_DIAGNOSTICS_TEMPLATE_ID));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.editor.runtime_diagnostics.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.editor.runtime_diagnostics.Open"));
    }
}
