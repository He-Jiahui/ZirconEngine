use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    AUDIO_ASSET_IMPORTER_DECLARATION, CODEC_IMPORTER_CAPABILITY, PLUGIN_ID, RUNTIME_CRATE_NAME,
};

pub const AUDIO_ASSET_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_asset_importer_audio_dist";
pub const AUDIO_ASSET_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_audio_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct AudioAssetImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl AudioAssetImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for AudioAssetImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for AudioAssetImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        package_manifest_from_descriptor(self.descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    AUDIO_ASSET_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(AudioAssetImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    AUDIO_ASSET_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = AUDIO_ASSET_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = AUDIO_ASSET_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    AUDIO_ASSET_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.audio.wav", ["wav"])
            .with_required_capabilities(["runtime.asset.importer.audio.wav"]),
        descriptor(
            "asset_importer.audio.codec",
            ["mp3", "ogg", "flac", "aif", "aiff"],
        )
        .with_required_capabilities([CODEC_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.audio.opus", ["opus"])
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
    manifest.supported_platforms = AUDIO_ASSET_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = AUDIO_ASSET_IMPORTER_DECLARATION
        .default_packaging()
        .to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        AUDIO_ASSET_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "asset_importer.audio.dist",
        AUDIO_ASSET_IMPORTER_DIST_CRATE_NAME,
        AUDIO_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Sound, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}
