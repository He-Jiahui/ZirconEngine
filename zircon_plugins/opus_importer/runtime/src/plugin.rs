use zircon_plugin_sdk::{
    importer_runtime_supported_platforms, importer_runtime_supported_targets,
    ImporterRuntimeManifestBuilder,
};
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter};
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use crate::{
    MISSING_BACKEND_DIAGNOSTIC, MODULE_NAME, NATIVE_IMPORTER_CAPABILITY, OPUS_IMPORTER_CAPABILITY,
    OPUS_IMPORTER_ID, OPUS_IMPORTER_PRIORITY, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};

pub const OPUS_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_opus_importer_dist";
pub const OPUS_IMPORTER_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_opus_importer_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct OpusImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl OpusImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for OpusImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for OpusImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        package_manifest_from_descriptor(self.descriptor())
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
            asset_importer_descriptor(),
            MISSING_BACKEND_DIAGNOSTIC,
        ))?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Opus Audio Importer",
        RuntimePluginId::OpusImporter,
        RUNTIME_CRATE_NAME,
    )
    .with_module_descriptor(module_descriptor())
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(OPUS_IMPORTER_CAPABILITY)
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(OpusImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, OPUS_IMPORTER_CAPABILITY]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    importer_runtime_supported_targets()
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    importer_runtime_supported_platforms()
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Opus audio importer plugin")
}

pub fn asset_importer_descriptor() -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(OPUS_IMPORTER_ID, PLUGIN_ID, AssetKind::Sound, 1)
        .with_priority(OPUS_IMPORTER_PRIORITY)
        .with_source_extensions(["opus"])
        .with_required_capabilities([OPUS_IMPORTER_CAPABILITY, NATIVE_IMPORTER_CAPABILITY])
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![asset_importer_descriptor()]
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    importer_manifest_builder().runtime_module_manifest()
}

pub fn dist_module_manifest() -> PluginModuleManifest {
    importer_manifest_builder().dist_module_manifest()
}

fn package_manifest_from_descriptor(descriptor: &RuntimePluginDescriptor) -> PluginPackageManifest {
    importer_manifest_builder()
        .with_asset_importers(asset_importer_descriptors())
        .build_package_manifest(descriptor)
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        "opus_importer.runtime",
        RUNTIME_CRATE_NAME,
        "opus_importer.dist",
        OPUS_IMPORTER_DIST_CRATE_NAME,
        OPUS_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}
