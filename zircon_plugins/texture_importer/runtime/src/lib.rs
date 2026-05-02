use image::GenericImageView;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAsset, TextureAsset,
    TexturePayload,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "texture_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_texture_importer_runtime";
pub const MODULE_NAME: &str = "TextureImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.texture_importer";
pub const IMAGE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.image";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMAGE_IMPORTER_CAPABILITY]
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
            "texture_importer.optional_container",
            90,
            ["psd", "dds", "ktx", "ktx2", "astc", "cubemap", "dxgi"],
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
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        if importer.id == "texture_importer.image" {
            registry.register_asset_importer(FunctionAssetImporter::new(importer, import_image))?;
        } else {
            registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                importer,
                "compressed texture containers require a NativeDynamic or VM texture backend",
            ))?;
        }
    }
    Ok(())
}

pub fn import_image(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let image = image::load_from_memory(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode image {}: {error}",
            context.source_path.display()
        ))
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    Ok(AssetImportOutcome::new(ImportedAsset::Texture(
        TextureAsset {
            uri: context.uri.clone(),
            width,
            height,
            rgba: rgba.into_raw(),
            payload: TexturePayload::Rgba8,
        },
    )))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_texture_importers() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"ktx2".to_string())));
    }

    #[test]
    fn registration_contributes_module_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 2);
    }

    #[test]
    fn image_importer_decodes_texture_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("checker.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "checker.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/checker.png").unwrap(),
            tiny_png_bytes(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2);
                assert_eq!(texture.height, 2);
                assert_eq!(texture.rgba.len(), 16);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    fn tiny_png_bytes() -> Vec<u8> {
        use image::{ImageBuffer, ImageFormat, Rgba};

        let image = ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            }
        });
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }
}
