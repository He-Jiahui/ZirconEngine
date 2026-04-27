use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = zircon_plugin_hybrid_gi_runtime::PLUGIN_ID;
pub const HYBRID_GI_AUTHORING_VIEW_ID: &str = "hybrid_gi.authoring";
pub const HYBRID_GI_DRAWER_ID: &str = "hybrid_gi.drawer";
pub const HYBRID_GI_TEMPLATE_ID: &str = "hybrid_gi.authoring";

#[derive(Clone, Debug)]
pub struct HybridGiEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl HybridGiEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for HybridGiEditorPlugin {
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
                drawer_id: HYBRID_GI_DRAWER_ID,
                drawer_display_name: "Hybrid GI Tools",
                template_id: HYBRID_GI_TEMPLATE_ID,
                template_document: "plugins://hybrid_gi/editor/authoring.ui.toml",
                surfaces: &[EditorAuthoringSurface::new(
                    HYBRID_GI_AUTHORING_VIEW_ID,
                    "Hybrid GI",
                    "Rendering",
                    "Plugins/Hybrid GI",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Hybrid GI",
        "zircon_plugin_hybrid_gi_editor",
    )
    .with_capability("editor.extension.hybrid_gi_authoring")
}

pub fn editor_plugin() -> HybridGiEditorPlugin {
    HybridGiEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_hybrid_gi_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_hybrid_gi_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_gi_editor_plugin_contributes_authoring_extensions() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&"editor.extension.hybrid_gi_authoring".to_string()));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == HYBRID_GI_AUTHORING_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == HYBRID_GI_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == HYBRID_GI_TEMPLATE_ID));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.hybrid_gi.authoring.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.hybrid_gi.authoring.Open"));
    }
}
