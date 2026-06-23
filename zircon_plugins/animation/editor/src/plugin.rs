use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::capability::{ANIMATION_AUTHORING_CAPABILITY, PLUGIN_ID};
use crate::extension_ids::{
    ANIMATION_AUTHORING_VIEW_ID, ANIMATION_DRAWER_ID, ANIMATION_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct AnimationEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl AnimationEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for AnimationEditorPlugin {
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
                drawer_id: ANIMATION_DRAWER_ID,
                drawer_display_name: "Animation Tools",
                template_id: ANIMATION_TEMPLATE_ID,
                template_document: "plugins://animation/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    ANIMATION_AUTHORING_VIEW_ID,
                    "Animation",
                    "World",
                    "Plugins/Animation",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Animation",
        "zircon_plugin_animation_editor",
    )
    .with_capability(ANIMATION_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> AnimationEditorPlugin {
    AnimationEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_animation_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_animation_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}
