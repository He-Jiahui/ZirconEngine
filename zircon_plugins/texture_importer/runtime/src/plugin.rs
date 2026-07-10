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
    import_cubemap_manifest, import_image, import_psd, import_texture_array_manifest,
    import_texture_container, ARRAY_IMPORTER_CAPABILITY, CONTAINER_IMPORTER_CAPABILITY,
    CUBEMAP_IMPORTER_CAPABILITY, IMAGE_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID,
    PSD_IMPORTER_CAPABILITY, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};

pub const TEXTURE_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_texture_importer_dist";
pub const TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_texture_importer_runtime_entry_v3";

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
        for importer in asset_importer_descriptors() {
            match importer.id.as_str() {
                "texture_importer.image" => registry
                    .register_asset_importer(FunctionAssetImporter::new(importer, import_image))?,
                "texture_importer.container" => registry.register_asset_importer(
                    FunctionAssetImporter::new(importer, import_texture_container),
                )?,
                "texture_importer.psd" => registry
                    .register_asset_importer(FunctionAssetImporter::new(importer, import_psd))?,
                "texture_importer.cubemap" => registry.register_asset_importer(
                    FunctionAssetImporter::new(importer, import_cubemap_manifest),
                )?,
                "texture_importer.array" => registry.register_asset_importer(
                    FunctionAssetImporter::new(importer, import_texture_array_manifest),
                )?,
                "texture_importer.optional_native_container" => {
                    registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                        importer,
                        "cubemap/dxgi texture import requires a NativeDynamic texture backend",
                    ))?;
                }
                _ => unreachable!(
                    "asset_importer_descriptors returns only known texture importer ids"
                ),
            }
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Texture Importer",
        RuntimePluginId::TextureImporter,
        RUNTIME_CRATE_NAME,
    )
    .with_module_descriptor(module_descriptor())
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(IMAGE_IMPORTER_CAPABILITY)
    .with_capability(CONTAINER_IMPORTER_CAPABILITY)
    .with_capability(PSD_IMPORTER_CAPABILITY)
    .with_capability(CUBEMAP_IMPORTER_CAPABILITY)
    .with_capability(ARRAY_IMPORTER_CAPABILITY)
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(TextureImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        IMAGE_IMPORTER_CAPABILITY,
        CONTAINER_IMPORTER_CAPABILITY,
        PSD_IMPORTER_CAPABILITY,
        CUBEMAP_IMPORTER_CAPABILITY,
        ARRAY_IMPORTER_CAPABILITY,
    ]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    importer_runtime_supported_targets()
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    importer_runtime_supported_platforms()
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Texture and image importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor(
            "texture_importer.image",
            120,
            [
                "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr",
                "qoi", "pnm", "pbm", "pgm", "ppm",
            ],
        )
        .with_required_capabilities([IMAGE_IMPORTER_CAPABILITY]),
        descriptor(
            "texture_importer.container",
            90,
            ["dds", "ktx", "ktx2", "astc"],
        )
        .with_required_capabilities([CONTAINER_IMPORTER_CAPABILITY]),
        descriptor("texture_importer.psd", 100, ["psd"])
            .with_required_capabilities([PSD_IMPORTER_CAPABILITY]),
        descriptor("texture_importer.cubemap", 130, ["zcube"])
            .with_required_capabilities([CUBEMAP_IMPORTER_CAPABILITY]),
        descriptor("texture_importer.array", 130, ["zarray"])
            .with_required_capabilities([ARRAY_IMPORTER_CAPABILITY]),
        descriptor(
            "texture_importer.optional_native_container",
            80,
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
    importer_manifest_builder()
        .with_asset_importers(asset_importer_descriptors())
        .build_package_manifest(descriptor)
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        "texture_importer.runtime",
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
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Texture, 1)
        .with_priority(priority)
        .with_source_extensions(extensions)
}
