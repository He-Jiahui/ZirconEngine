use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::capability::{PLUGIN_ID, TEXTURE_AUTHORING_CAPABILITY};
use crate::extension_ids::{TEXTURE_AUTHORING_VIEW_ID, TEXTURE_DRAWER_ID, TEXTURE_TEMPLATE_ID};

#[derive(Clone, Debug)]
pub struct TextureEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl TextureEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for TextureEditorPlugin {
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
                drawer_id: TEXTURE_DRAWER_ID,
                drawer_display_name: "Texture Tools",
                template_id: TEXTURE_TEMPLATE_ID,
                template_document: "plugins://texture/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    TEXTURE_AUTHORING_VIEW_ID,
                    "Texture",
                    "Assets",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Texture", "zircon_plugin_texture_editor")
        .with_capability(TEXTURE_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> TextureEditorPlugin {
    TextureEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_texture_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_texture_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
