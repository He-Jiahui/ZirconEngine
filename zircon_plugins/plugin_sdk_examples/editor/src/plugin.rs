use zircon_editor::core::editor_extension::{
    EditorExtensionRegistry, EditorExtensionRegistryError,
};
use zircon_plugin_sdk::editor::authoring_plugin;
use zircon_plugin_sdk::prelude::PluginMaturity;
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, PluginDistributionManifest, PluginModuleManifest,
    PluginPackageManifest,
};

use crate::capability::{
    ASSET_FIXTURE_CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, WINDOW_CAPABILITY,
};
use crate::extensions::{register_example_window, register_importer_and_inspector};

const DISPLAY_NAME: &str = "Plugin SDK Examples";
const CRATE_NAME: &str = "zircon_plugin_sdk_examples_editor";
pub const PLUGIN_SDK_EXAMPLES_DIST_CRATE_NAME: &str = "zircon_plugin_sdk_examples_dist";
pub const PLUGIN_SDK_EXAMPLES_DIST_EDITOR_ENTRY: &str =
    "zircon_plugin_sdk_examples_editor_entry_v3";
const PLUGIN_SDK_EXAMPLES_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

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
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_package_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    editor_plugin().editor_capabilities()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        base_package_manifest(),
    )
}

fn base_package_manifest() -> PluginPackageManifest {
    editor_plugin()
        .declaration()
        .base_manifest()
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
        .with_native_module(plugin_sdk_examples_dist_module_manifest())
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: PLUGIN_SDK_EXAMPLES_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: PLUGIN_SDK_EXAMPLES_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            editor_entry: PLUGIN_SDK_EXAMPLES_DIST_EDITOR_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
}

pub fn plugin_sdk_examples_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native(
        "plugin_sdk_examples.dist",
        PLUGIN_SDK_EXAMPLES_DIST_CRATE_NAME,
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
}
