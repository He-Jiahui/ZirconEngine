use zircon_plugin_sdk::{
    importer_runtime_supported_platforms, importer_runtime_supported_targets,
    ImporterRuntimeManifestBuilder,
};
use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    ExportTargetPlatform, PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    import_symphonia_audio, import_wav, CODEC_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME, WAV_IMPORTER_CAPABILITY,
};

pub const AUDIO_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_audio_importer_dist";
pub const AUDIO_IMPORTER_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_audio_importer_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct AudioImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl AudioImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for AudioImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for AudioImporterRuntimePlugin {
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
        for importer in asset_importer_descriptors() {
            match importer.id.as_str() {
                "audio_importer.wav" => registry
                    .register_asset_importer(FunctionAssetImporter::new(importer, import_wav))?,
                "audio_importer.codec" => registry.register_asset_importer(
                    FunctionAssetImporter::new(importer, import_symphonia_audio),
                )?,
                "audio_importer.opus" => {
                    registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                        importer,
                        "opus import requires a NativeDynamic libopus backend",
                    ))?;
                }
                _ => {
                    unreachable!("asset_importer_descriptors returns only known audio importer ids")
                }
            }
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Audio Importer",
        RuntimePluginId::AudioImporter,
        RUNTIME_CRATE_NAME,
    )
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(WAV_IMPORTER_CAPABILITY)
    .with_capability(CODEC_IMPORTER_CAPABILITY)
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(AudioImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        WAV_IMPORTER_CAPABILITY,
        CODEC_IMPORTER_CAPABILITY,
    ]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    importer_runtime_supported_targets()
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    importer_runtime_supported_platforms()
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Audio importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("audio_importer.wav", 120, ["wav"])
            .with_required_capabilities([WAV_IMPORTER_CAPABILITY]),
        descriptor(
            "audio_importer.codec",
            90,
            ["mp3", "ogg", "flac", "aif", "aiff"],
        )
        .with_required_capabilities([CODEC_IMPORTER_CAPABILITY]),
        descriptor("audio_importer.opus", 80, ["opus"])
            .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
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
        "audio_importer.runtime",
        RUNTIME_CRATE_NAME,
        "audio_importer.dist",
        AUDIO_IMPORTER_DIST_CRATE_NAME,
        AUDIO_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Sound, 1)
        .with_priority(priority)
        .with_source_extensions(extensions)
}
