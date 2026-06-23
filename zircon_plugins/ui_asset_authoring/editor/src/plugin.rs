use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::{
    CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, UI_ASSET_DRAWER_ID, UI_ASSET_TEMPLATE_ID,
    UI_ASSET_VIEW_ID,
};

#[derive(Clone, Debug)]
pub struct UiAssetAuthoringEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl UiAssetAuthoringEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for UiAssetAuthoringEditorPlugin {
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
                drawer_id: UI_ASSET_DRAWER_ID,
                drawer_display_name: "UI Asset Tools",
                template_id: UI_ASSET_TEMPLATE_ID,
                template_document: "plugins://ui_asset_authoring/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    UI_ASSET_VIEW_ID,
                    "UI Asset",
                    "Assets",
                    "Plugins/UI Asset",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "UI Asset Authoring",
        "zircon_plugin_ui_asset_authoring_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> UiAssetAuthoringEditorPlugin {
    UiAssetAuthoringEditorPlugin::new()
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
    zircon_runtime::plugin::PluginPackageManifest::new(PLUGIN_ID, "UI Asset Authoring")
        .with_category("authoring")
        .with_supported_targets([zircon_runtime::builtin::RuntimeTargetMode::EditorHost])
        .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
}
