use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    import_symphonia_audio, import_wav, AUDIO_IMPORTER_DECLARATION, CODEC_IMPORTER_CAPABILITY,
    PLUGIN_ID, RUNTIME_CRATE_NAME, WAV_IMPORTER_CAPABILITY,
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
    AUDIO_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(AudioImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    AUDIO_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = AUDIO_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = AUDIO_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    AUDIO_IMPORTER_DECLARATION.module_descriptor()
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
    let mut manifest = importer_manifest_builder()
        .with_asset_importers(asset_importer_descriptors())
        .build_package_manifest(descriptor);
    manifest.supported_platforms = AUDIO_IMPORTER_DECLARATION.supported_platforms().to_vec();
    manifest.default_packaging = AUDIO_IMPORTER_DECLARATION.default_packaging().to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        AUDIO_IMPORTER_DECLARATION.module_name(),
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
