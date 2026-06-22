use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_plugin_sdk::editor::authoring_plugin;
use zircon_plugin_sdk::prelude::PluginMaturity;
use zircon_runtime::plugin::PluginPackageManifest;

use crate::capability::{
    ASSET_FIXTURE_CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, WINDOW_CAPABILITY,
};
use crate::extensions::{register_example_window, register_importer_and_inspector};

const DISPLAY_NAME: &str = "Plugin SDK Examples";
const CRATE_NAME: &str = "zircon_plugin_sdk_examples_editor";

authoring_plugin! {
    pub struct PluginSdkExamplesEditorPlugin {
        package_id: PLUGIN_ID,
        display_name: DISPLAY_NAME,
        crate_name: CRATE_NAME,
        category: "sdk",
        description: "Editor SDK fixture package containing a window plugin and an importer plus inspector plugin.",
        maturity: PluginMaturity::Experimental,
        capabilities: EDITOR_CAPABILITIES,
        asset_root: "assets",
        content_root: "examples",
        register_extensions: register_editor_package_extensions,
    }
}

fn register_editor_package_extensions(
    registry: &mut EditorExtensionRegistry,
) -> Result<(), EditorExtensionRegistryError> {
    register_example_window(registry)?;
    register_importer_and_inspector(registry)
}

#[derive(Clone, Debug)]
pub struct ExampleWindowEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl Default for ExampleWindowEditorPlugin {
    fn default() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                "plugin_sdk_examples.window",
                "SDK Example Window",
                CRATE_NAME,
            )
            .with_capability(WINDOW_CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for ExampleWindowEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_example_window(registry)
    }
}

#[derive(Clone, Debug)]
pub struct ExampleAssetInspectorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl Default for ExampleAssetInspectorPlugin {
    fn default() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                "plugin_sdk_examples.asset",
                "SDK Example Asset Tools",
                CRATE_NAME,
            )
            .with_capability(ASSET_FIXTURE_CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for ExampleAssetInspectorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        register_importer_and_inspector(registry)
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    editor_plugin().declaration().descriptor().clone()
}

pub fn editor_plugin() -> PluginSdkExamplesEditorPlugin {
    PluginSdkExamplesEditorPlugin::new()
}

pub fn package_manifest() -> PluginPackageManifest {
    editor_plugin().package_manifest()
}

pub fn editor_capabilities() -> Vec<String> {
    editor_plugin().editor_capabilities()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    editor_plugin().registration_report()
}
