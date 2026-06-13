use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

use crate::{
    import_image, import_psd, import_texture_container, CONTAINER_IMPORTER_CAPABILITY,
    IMAGE_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID, PSD_IMPORTER_CAPABILITY, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME,
};

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        IMAGE_IMPORTER_CAPABILITY,
        CONTAINER_IMPORTER_CAPABILITY,
        PSD_IMPORTER_CAPABILITY,
    ]
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
        descriptor(
            "texture_importer.optional_native_container",
            80,
            ["cubemap", "dxgi"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Texture Importer")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("texture_importer.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        match importer.id.as_str() {
            "texture_importer.image" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_image))?,
            "texture_importer.container" => registry.register_asset_importer(
                FunctionAssetImporter::new(importer, import_texture_container),
            )?,
            "texture_importer.psd" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_psd))?,
            "texture_importer.optional_native_container" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "cubemap/dxgi texture import requires a NativeDynamic texture backend",
                ))?;
            }
            _ => unreachable!("asset_importer_descriptors returns only known texture importer ids"),
        }
    }
    Ok(())
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
