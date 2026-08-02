use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    CONTAINER_IMPORTER_CAPABILITY, PLUGIN_ID, PSD_IMPORTER_CAPABILITY, RUNTIME_CRATE_NAME,
    TEXTURE_ASSET_IMPORTER_DECLARATION,
};

pub const TEXTURE_ASSET_IMPORTER_DIST_CRATE_NAME: &str =
    "zircon_plugin_asset_importer_texture_dist";
pub const TEXTURE_ASSET_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_texture_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct TextureAssetImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl TextureAssetImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for TextureAssetImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for TextureAssetImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        package_manifest_from_descriptor(self.descriptor())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    TEXTURE_ASSET_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(TextureAssetImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    TEXTURE_ASSET_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = TEXTURE_ASSET_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = TEXTURE_ASSET_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    TEXTURE_ASSET_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor(
            "asset_importer.texture.image",
            [
                "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr",
                "qoi", "pnm", "pbm", "pgm", "ppm",
            ],
        )
        .with_required_capabilities(["runtime.asset.importer.texture.image"]),
        descriptor(
            "asset_importer.texture.container",
            ["dds", "ktx", "ktx2", "astc"],
        )
        .with_required_capabilities([CONTAINER_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.texture.psd", ["psd"])
            .with_required_capabilities([PSD_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.texture.optional_native_container",
            ["cubemap", "dxgi"],
        )
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
    manifest.supported_platforms = TEXTURE_ASSET_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = TEXTURE_ASSET_IMPORTER_DECLARATION
        .default_packaging()
        .to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        TEXTURE_ASSET_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "asset_importer.texture.dist",
        TEXTURE_ASSET_IMPORTER_DIST_CRATE_NAME,
        TEXTURE_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Texture, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}
