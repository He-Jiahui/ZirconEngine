use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    ExportTargetPlatform, PluginModuleManifest, PluginPackageManifest, ProjectPluginSelection,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use crate::{
    MISSING_BACKEND_DIAGNOSTIC, MODULE_NAME, NATIVE_IMPORTER_CAPABILITY, OPUS_IMPORTER_CAPABILITY,
    OPUS_IMPORTER_ID, OPUS_IMPORTER_PRIORITY, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};

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
        registry.register_module(module_descriptor())?;
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
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(OPUS_IMPORTER_CAPABILITY)
    .build()
}

pub fn runtime_plugin() -> OpusImporterRuntimePlugin {
    OpusImporterRuntimePlugin::new()
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, OPUS_IMPORTER_CAPABILITY]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
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

pub fn package_manifest() -> PluginPackageManifest {
    RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("opus_importer.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

fn package_manifest_from_descriptor(descriptor: &RuntimePluginDescriptor) -> PluginPackageManifest {
    let mut manifest = descriptor.package_manifest();
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}
