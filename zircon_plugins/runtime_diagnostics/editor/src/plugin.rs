use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportTargetPlatform, plugin::PluginDistributionManifest,
    plugin::PluginModuleManifest, plugin::PluginPackageManifest,
};

use crate::{
    CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID, RUNTIME_DIAGNOSTICS_DRAWER_ID,
    RUNTIME_DIAGNOSTICS_TEMPLATE_ID, RUNTIME_DIAGNOSTICS_VIEW_ID,
};

pub const RUNTIME_DIAGNOSTICS_DIST_CRATE_NAME: &str = "zircon_plugin_runtime_diagnostics_dist";
pub const RUNTIME_DIAGNOSTICS_DIST_EDITOR_ENTRY: &str =
    "zircon_plugin_runtime_diagnostics_editor_entry_v3";
const RUNTIME_DIAGNOSTICS_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

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
                template_document: "plugins://runtime_diagnostics/editor/authoring.zui",
                surfaces: &[EditorAuthoringSurface::new(
                    RUNTIME_DIAGNOSTICS_VIEW_ID,
                    "Runtime Diagnostics",
                    "Diagnostics",
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
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> RuntimeDiagnosticsEditorPlugin {
    RuntimeDiagnosticsEditorPlugin::new()
}

pub fn package_manifest() -> PluginPackageManifest {
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

fn base_package_manifest() -> PluginPackageManifest {
    PluginPackageManifest::new(PLUGIN_ID, "Runtime Diagnostics")
        .with_category("diagnostics")
        .with_supported_targets([RuntimeTargetMode::EditorHost])
        .with_supported_platforms([
            ExportTargetPlatform::Windows,
            ExportTargetPlatform::Linux,
            ExportTargetPlatform::Macos,
        ])
        .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
        .with_native_module(runtime_diagnostics_dist_module_manifest())
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: RUNTIME_DIAGNOSTICS_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: RUNTIME_DIAGNOSTICS_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            editor_entry: RUNTIME_DIAGNOSTICS_DIST_EDITOR_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
}

pub fn runtime_diagnostics_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native(
        "runtime_diagnostics.dist",
        RUNTIME_DIAGNOSTICS_DIST_CRATE_NAME,
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capabilities(EDITOR_CAPABILITIES.iter().copied())
}
