use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::capability::{HYBRID_GI_AUTHORING_CAPABILITY, PLUGIN_ID};
use crate::extension_ids::{
    HYBRID_GI_AUTHORING_VIEW_ID, HYBRID_GI_DRAWER_ID, HYBRID_GI_TEMPLATE_ID,
};

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
                template_document: "plugins://hybrid_gi/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    HYBRID_GI_AUTHORING_VIEW_ID,
                    "Hybrid GI",
                    "Rendering",
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
    .with_capability(HYBRID_GI_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> HybridGiEditorPlugin {
    HybridGiEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
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
