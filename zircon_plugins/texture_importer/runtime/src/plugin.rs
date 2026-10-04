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
    import_cubemap, import_image, import_psd, import_texture_array_manifest,
    import_texture_container, ARRAY_IMPORTER_CAPABILITY, CONTAINER_IMPORTER_CAPABILITY,
    CUBEMAP_IMPORTER_CAPABILITY, IMAGE_IMPORTER_CAPABILITY, PLUGIN_ID, PSD_IMPORTER_CAPABILITY,
    RUNTIME_CRATE_NAME, TEXTURE_IMPORTER_DECLARATION,
};

pub const TEXTURE_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_texture_importer_dist";
pub const TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_texture_importer_runtime_entry_v3";
const TEXTURE_IMPORTER_VERSION: u32 = 2;
// KTX2 output/admission changes must invalidate previously accepted containers.
const CONTAINER_IMPORTER_VERSION: u32 = 3;
const CUBEMAP_IMPORTER_VERSION: u32 = 3;

#[derive(Clone, Debug)]
pub struct TextureImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl TextureImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for TextureImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for TextureImporterRuntimePlugin {
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
        registry.register_asset_importer(FunctionAssetImporter::new(
            image_importer_descriptor(),
            import_image,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            container_importer_descriptor(),
            import_texture_container,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            psd_importer_descriptor(),
            import_psd,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            cubemap_importer_descriptor(),
            import_cubemap,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            array_importer_descriptor(),
            import_texture_array_manifest,
        ))?;
        registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
            optional_native_container_importer_descriptor(),
            "cubemap/dxgi texture import requires a NativeDynamic texture backend",
        ))?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    TEXTURE_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(TextureImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    TEXTURE_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = TEXTURE_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = TEXTURE_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    TEXTURE_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        image_importer_descriptor(),
        container_importer_descriptor(),
        psd_importer_descriptor(),
        cubemap_importer_descriptor(),
        array_importer_descriptor(),
        optional_native_container_importer_descriptor(),
    ]
}

fn image_importer_descriptor() -> AssetImporterDescriptor {
    descriptor(
        "texture_importer.image",
        120,
        [
            "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr", "qoi",
            "pnm", "pbm", "pgm", "ppm",
        ],
    )
    .with_required_capabilities([IMAGE_IMPORTER_CAPABILITY])
}

fn container_importer_descriptor() -> AssetImporterDescriptor {
    descriptor_with_version(
        "texture_importer.container",
        CONTAINER_IMPORTER_VERSION,
        90,
        ["dds", "ktx", "ktx2", "astc"],
    )
    .with_required_capabilities([CONTAINER_IMPORTER_CAPABILITY])
}

fn psd_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("texture_importer.psd", 100, ["psd"])
        .with_required_capabilities([PSD_IMPORTER_CAPABILITY])
}

fn cubemap_importer_descriptor() -> AssetImporterDescriptor {
    descriptor_with_version(
        "texture_importer.cubemap",
        CUBEMAP_IMPORTER_VERSION,
        130,
        ["zcube"],
    )
    .with_required_capabilities([CUBEMAP_IMPORTER_CAPABILITY])
}

fn array_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("texture_importer.array", 130, ["zarray"])
        .with_required_capabilities([ARRAY_IMPORTER_CAPABILITY])
}

fn optional_native_container_importer_descriptor() -> AssetImporterDescriptor {
    descriptor(
        "texture_importer.optional_native_container",
        80,
        ["cubemap", "dxgi"],
    )
    .with_required_capabilities(["runtime.asset.importer.native"])
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
    manifest.supported_platforms = TEXTURE_IMPORTER_DECLARATION.supported_platforms().to_vec();
    manifest.default_packaging = TEXTURE_IMPORTER_DECLARATION.default_packaging().to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        TEXTURE_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "texture_importer.dist",
        TEXTURE_IMPORTER_DIST_CRATE_NAME,
        TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    descriptor_with_version(id, TEXTURE_IMPORTER_VERSION, priority, extensions)
}

fn descriptor_with_version(
    id: impl Into<String>,
    importer_version: u32,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Texture, importer_version)
        .with_priority(priority)
        .with_source_extensions(extensions)
}

#[cfg(test)]
#[path = "tests/plugin.rs"]
mod tests;
