use crate::plugin_registration;

use super::support::{
    tiny_image_bytes, tiny_jpeg_bytes, tiny_png_bytes, tiny_rgb32f_image_bytes,
    tiny_stacked_png_bytes,
};

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
fn image_importer_rejects_invalid_texture_metadata() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("normal.png"))
        .expect("png importer");
    let context = zircon_runtime::asset::AssetImportContext::new(
        "normal.png".into(),
        zircon_runtime::asset::AssetUri::parse("res://textures/normal.png").unwrap(),
        tiny_png_bytes(),
        r#"usage_hint = "normal"
color_space = "srgb""#
            .parse()
            .expect("valid texture import settings"),
    );

    let error = importer
        .import(&context)
        .expect_err("normal map metadata must be linear");

    assert!(
        error
            .to_string()
            .contains("normal map must use linear color space"),
        "unexpected error: {error}"
    );
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
                vec![zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld,]
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
                vec![zircon_runtime::core::framework::render::RenderImageAssetUsage::RenderWorld]
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
