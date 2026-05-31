mod container;
use container::parse_container_info;
use zircon_runtime::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
    ImportedAsset, TextureAsset, TextureAssetDescriptor,
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
pub const CONTAINER_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.container";
pub const PSD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.psd";

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

pub fn import_image(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let texture = apply_texture_import_settings(
        context,
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba),
    )?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

pub fn import_psd(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let psd = psd::Psd::from_bytes(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode psd {}: {error}",
            context.source_path.display()
        ))
    })?;
    let width = psd.width();
    let height = psd.height();
    let rgba = psd.rgba();
    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return Err(AssetImportError::Parse(format!(
            "decode psd {}: decoded rgba length {} did not match expected {}",
            context.source_path.display(),
            rgba.len(),
            expected_len
        )));
    }

    let texture = apply_texture_import_settings(
        context,
        TextureAsset::new_rgba8(context.uri.clone(), width, height, rgba),
    )?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

pub fn import_texture_container(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let info = parse_container_info(context)?;
    let mut descriptor =
        TextureAssetDescriptor::container(info.format.clone(), info.mip_count, info.array_layers);
    descriptor.dimension = info.dimension;
    descriptor.depth_or_array_layers = info.depth_or_array_layers;
    let texture = apply_texture_import_settings(
        context,
        TextureAsset::new_container(
            context.uri.clone(),
            info.width,
            info.height,
            info.format,
            context.source_bytes.clone(),
            info.mip_count,
            info.array_layers,
        )
        .with_descriptor(descriptor),
    )?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

fn apply_texture_import_settings(
    context: &AssetImportContext,
    texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    texture
        .with_import_settings(&context.import_settings)
        .map_err(|error| {
            AssetImportError::Parse(format!(
                "apply texture import settings {}: {error}",
                context.source_path.display()
            ))
        })
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
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 4);
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

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2);
                assert_eq!(texture.height, 2);
                assert_eq!(texture.rgba.len(), 16);
                assert_eq!(
                    texture.render_image_descriptor().format,
                    zircon_runtime::asset::RGBA8_UNORM_SRGB_FORMAT
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_decodes_common_extension_format_matrix() {
        let report = plugin_registration();
        let cases = [
            ("swatch.bmp", image::ImageFormat::Bmp),
            ("swatch.tga", image::ImageFormat::Tga),
            ("swatch.tiff", image::ImageFormat::Tiff),
            ("swatch.gif", image::ImageFormat::Gif),
            ("swatch.webp", image::ImageFormat::WebP),
            ("swatch.hdr", image::ImageFormat::Hdr),
            ("swatch.exr", image::ImageFormat::OpenExr),
            ("swatch.qoi", image::ImageFormat::Qoi),
            ("swatch.ppm", image::ImageFormat::Pnm),
        ];

        for (path, format) in cases {
            let importer = report
                .extensions
                .asset_importers()
                .select(std::path::Path::new(path))
                .unwrap();
            let uri = format!("res://textures/{path}");
            let context = zircon_runtime::asset::AssetImportContext::new(
                path.into(),
                zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
                tiny_image_bytes(format),
                Default::default(),
            );

            let outcome = importer.import(&context).unwrap();
            let imported = &outcome
                .root_entry()
                .expect("root texture asset entry")
                .asset;

            match imported {
                zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                    assert_eq!(texture.width, 2, "{path}");
                    assert_eq!(texture.height, 2, "{path}");
                    assert_eq!(texture.rgba.len(), 16, "{path}");
                }
                other => panic!("unexpected imported asset for {path}: {other:?}"),
            }
        }
    }

    #[test]
    fn image_importer_uses_extension_format_by_default() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("mismatched.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "mismatched.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/mismatched.png").unwrap(),
            tiny_jpeg_bytes(),
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err().to_string();

        assert!(
            error.contains("decode image as `png` from extension"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn image_importer_can_guess_format_from_bytes_when_requested() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("mismatched.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "mismatched.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/mismatched.png").unwrap(),
            tiny_jpeg_bytes(),
            r#"image_format = "guess""#
                .parse()
                .expect("valid image import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2);
                assert_eq!(texture.height, 2);
                assert_eq!(texture.rgba.len(), 16);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_can_use_explicit_source_format() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("mismatched.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "mismatched.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/mismatched.png").unwrap(),
            tiny_jpeg_bytes(),
            r#"image_format = "jpeg""#
                .parse()
                .expect("valid image import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2);
                assert_eq!(texture.height, 2);
                assert_eq!(texture.rgba.len(), 16);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_accepts_source_format_aliases() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("mismatched.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "mismatched.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/mismatched.png").unwrap(),
            tiny_rgb32f_image_bytes(image::ImageFormat::OpenExr),
            r#"source_format = "open_exr""#
                .parse()
                .expect("valid image import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2);
                assert_eq!(texture.height, 2);
                assert_eq!(texture.rgba.len(), 16);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_reports_actual_source_format_key() {
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
            "decode_format = 1"
                .parse()
                .expect("valid image import settings"),
        );

        let error = importer.import(&context).unwrap_err().to_string();

        assert!(
            error.contains("image import setting `decode_format` must be a string"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn image_importer_applies_texture_descriptor_settings() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("height.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "height.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/height.png").unwrap(),
            tiny_png_bytes(),
            r#"
format = "rgba16float"
color_space = "linear"
dimension = "3d"
usage = ["sampled", "storage"]
asset_usage = ["render_world"]
mip_count = 2
depth_or_array_layers = 4

[sampler]
address_mode_u = "repeat"
mag_filter = "nearest"
"#
            .parse()
            .expect("valid texture import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "rgba16float");
                assert_eq!(
                    descriptor.color_space,
                    zircon_runtime::core::framework::render::RenderImageColorSpace::Linear
                );
                assert_eq!(
                    descriptor.dimension,
                    zircon_runtime::core::framework::render::RenderImageDimension::D3
                );
                assert_eq!(
                    descriptor.usage,
                    vec![
                        zircon_runtime::core::framework::render::RenderImageUsage::Sampled,
                        zircon_runtime::core::framework::render::RenderImageUsage::Storage,
                    ]
                );
                assert_eq!(
                    descriptor.asset_usage,
                    vec![
                        zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld,
                    ]
                );
                assert_eq!(descriptor.mip_count, 2);
                assert_eq!(descriptor.depth_or_array_layers, 4);
                assert_eq!(
                    descriptor.sampler.address_mode_u,
                    zircon_runtime::core::framework::render::RenderSamplerAddressMode::Repeat
                );
                assert_eq!(
                    descriptor.sampler.mag_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_accepts_bevy_image_setting_aliases() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("linear.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "linear.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/linear.png").unwrap(),
            tiny_png_bytes(),
            r#"
texture_format = "rgba16float"
is_srgb = false
sampler = "nearest"
asset_usage = "render_world"
"#
            .parse()
            .expect("valid texture import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "rgba16float");
                assert_eq!(
                    descriptor.color_space,
                    zircon_runtime::core::framework::render::RenderImageColorSpace::Linear
                );
                assert_eq!(
                    descriptor.sampler.mag_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.sampler.min_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.sampler.mipmap_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.asset_usage,
                    vec![
                        zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld
                    ]
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_normalizes_default_linear_rgba8_format() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("linear-default.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "linear-default.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/linear-default.png").unwrap(),
            tiny_png_bytes(),
            r#"is_srgb = false"#.parse().expect("valid texture import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, zircon_runtime::asset::RGBA8_UNORM_FORMAT);
                assert_eq!(
                    descriptor.color_space,
                    zircon_runtime::core::framework::render::RenderImageColorSpace::Linear
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn image_importer_reinterprets_stacked_array_layout() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("stacked.png"))
            .unwrap();
        for (layout_name, settings) in [
            (
                "row_count",
                r#"
[array_layout]
row_count = 2
"#,
            ),
            (
                "row_height",
                r#"
[array_layout]
row_height = 2
"#,
            ),
        ] {
            let context = zircon_runtime::asset::AssetImportContext::new(
                "stacked.png".into(),
                zircon_runtime::asset::AssetUri::parse("res://textures/stacked.png").unwrap(),
                tiny_stacked_png_bytes(),
                settings.parse().expect("valid texture import settings"),
            );

            let outcome = importer.import(&context).unwrap();
            let imported = &outcome
                .root_entry()
                .expect("root texture asset entry")
                .asset;

            match imported {
                zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                    assert_eq!(texture.width, 2, "{layout_name}");
                    assert_eq!(texture.height, 2, "{layout_name}");
                    assert_eq!(texture.rgba.len(), 2 * 4 * 4, "{layout_name}");
                    let descriptor = texture.render_image_descriptor();
                    assert_eq!(descriptor.height, 2, "{layout_name}");
                    assert_eq!(descriptor.array_layer_count, 2, "{layout_name}");
                    assert_eq!(descriptor.depth_or_array_layers, 2, "{layout_name}");
                    assert_eq!(
                        descriptor.dimension,
                        zircon_runtime::core::framework::render::RenderImageDimension::D2,
                        "{layout_name}"
                    );
                }
                other => panic!("unexpected imported asset for {layout_name}: {other:?}"),
            }
        }
    }

    #[test]
    fn image_importer_rejects_invalid_array_layout() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("stacked.png"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "stacked.png".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/stacked.png").unwrap(),
            tiny_stacked_png_bytes(),
            r#"
[array_layout]
row_count = 3
"#
            .parse()
            .expect("valid texture import settings"),
        );

        let error = importer.import(&context).unwrap_err().to_string();

        assert!(
            error.contains("can not evenly divide height = 4 by layers = 3"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn psd_importer_decodes_flattened_rgba_texture_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("swatch.psd"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "swatch.psd".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/swatch.psd").unwrap(),
            tiny_psd_bytes(),
            Default::default(),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 1);
                assert_eq!(texture.height, 1);
                assert_eq!(texture.rgba, vec![12, 34, 56, 200]);
                assert_eq!(
                    texture.payload,
                    zircon_runtime::asset::TexturePayload::Rgba8
                );
                assert!(texture.descriptor.is_some());
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn psd_importer_applies_texture_descriptor_settings() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("swatch.psd"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "swatch.psd".into(),
            zircon_runtime::asset::AssetUri::parse("res://textures/swatch.psd").unwrap(),
            tiny_psd_bytes(),
            r#"
texture_format = "rgba16float"
is_srgb = false
sampler = "nearest"
asset_usage = "render_world"
"#
            .parse()
            .expect("valid texture import settings"),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome
            .root_entry()
            .expect("root texture asset entry")
            .asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Texture(texture) => {
                assert_eq!(texture.rgba, vec![12, 34, 56, 200]);
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.format, "rgba16float");
                assert_eq!(
                    descriptor.color_space,
                    zircon_runtime::core::framework::render::RenderImageColorSpace::Linear
                );
                assert_eq!(
                    descriptor.sampler.mag_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.sampler.min_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.sampler.mipmap_filter,
                    zircon_runtime::core::framework::render::RenderSamplerFilter::Nearest
                );
                assert_eq!(
                    descriptor.asset_usage,
                    vec![
                        zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld
                    ]
                );
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

    fn tiny_stacked_png_bytes() -> Vec<u8> {
        use image::{ImageBuffer, ImageFormat, Rgba};

        let image = ImageBuffer::<Rgba<u8>, _>::from_fn(2, 4, |_x, y| {
            if y < 2 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        });
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    fn tiny_jpeg_bytes() -> Vec<u8> {
        use image::{ImageBuffer, ImageFormat, Rgb};

        let image = ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255, 0, 0])
            } else {
                Rgb([0, 0, 255])
            }
        });
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Jpeg).unwrap();
        bytes.into_inner()
    }

    fn tiny_image_bytes(format: image::ImageFormat) -> Vec<u8> {
        if matches!(
            format,
            image::ImageFormat::Hdr | image::ImageFormat::OpenExr
        ) {
            return tiny_rgb32f_image_bytes(format);
        }

        use image::{DynamicImage, ImageBuffer, Rgb};

        let image = ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([255, 255, 255])
            } else {
                Rgb([0, 0, 0])
            }
        });
        let dynamic = DynamicImage::ImageRgb8(image);
        let mut bytes = std::io::Cursor::new(Vec::new());
        dynamic.write_to(&mut bytes, format).unwrap();
        bytes.into_inner()
    }

    fn tiny_rgb32f_image_bytes(format: image::ImageFormat) -> Vec<u8> {
        use image::{DynamicImage, ImageBuffer, Rgb};

        let image = ImageBuffer::<Rgb<f32>, _>::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgb([1.0, 0.25, 0.0])
            } else {
                Rgb([0.0, 0.5, 1.0])
            }
        });
        let dynamic = DynamicImage::ImageRgb32F(image);
        let mut bytes = std::io::Cursor::new(Vec::new());
        dynamic.write_to(&mut bytes, format).unwrap();
        bytes.into_inner()
    }

    fn tiny_psd_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"8BPS");
        bytes.extend_from_slice(&1_u16.to_be_bytes());
        bytes.extend_from_slice(&[0; 6]);
        bytes.extend_from_slice(&4_u16.to_be_bytes());
        bytes.extend_from_slice(&1_u32.to_be_bytes());
        bytes.extend_from_slice(&1_u32.to_be_bytes());
        bytes.extend_from_slice(&8_u16.to_be_bytes());
        bytes.extend_from_slice(&3_u16.to_be_bytes());
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes.extend_from_slice(&0_u16.to_be_bytes());
        bytes.extend_from_slice(&[12, 34, 56, 200]);
        bytes
    }
}
