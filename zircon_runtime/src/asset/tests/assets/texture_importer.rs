use std::fs;

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, Rgba};

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::importer_with_first_wave_plugin_fixtures;
use crate::asset::{AssetUri, ImportedAsset, TextureUploadSupport, RGBA8_UNORM_FORMAT};
use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDimension, RenderImageUsage,
    RenderSamplerAddressMode, RenderSamplerFilter,
};

#[test]
fn importer_decodes_png_and_jpeg_textures() {
    let root = unique_temp_project_root("texture_import");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("checker.png");
    let jpg_path = root.join("checker.jpg");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    })
    .save_with_format(&png_path, ImageFormat::Png)
    .unwrap();

    ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([255, 0, 0])
        } else {
            Rgb([0, 0, 255])
        }
    })
    .save_with_format(&jpg_path, ImageFormat::Jpeg)
    .unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();
    let png = importer
        .import_from_source(
            &png_path,
            &AssetUri::parse("res://textures/checker.png").unwrap(),
        )
        .unwrap();
    let jpg = importer
        .import_from_source(
            &jpg_path,
            &AssetUri::parse("res://textures/checker.jpg").unwrap(),
        )
        .unwrap();

    match png {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
            assert_eq!(
                texture.render_image_descriptor().format,
                crate::asset::RGBA8_UNORM_SRGB_FORMAT
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    match jpg {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
            assert_eq!(texture.render_image_descriptor().mip_count, 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_decodes_common_extension_format_matrix() {
    let root = unique_temp_project_root("texture_import_format_matrix");
    fs::create_dir_all(&root).unwrap();
    let cases = [
        ("swatch.bmp", ImageFormat::Bmp),
        ("swatch.tga", ImageFormat::Tga),
        ("swatch.tiff", ImageFormat::Tiff),
        ("swatch.gif", ImageFormat::Gif),
        ("swatch.webp", ImageFormat::WebP),
        ("swatch.hdr", ImageFormat::Hdr),
        ("swatch.exr", ImageFormat::OpenExr),
        ("swatch.qoi", ImageFormat::Qoi),
        ("swatch.ppm", ImageFormat::Pnm),
    ];

    let importer = importer_with_first_wave_plugin_fixtures();
    for (name, format) in cases {
        let path = root.join(name);
        fs::write(&path, tiny_rgb_image_bytes(format)).unwrap();
        let uri = AssetUri::parse(&format!("res://textures/{name}")).unwrap();

        let imported = importer.import_from_source(&path, &uri).unwrap();

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2, "{name}");
                assert_eq!(texture.height, 2, "{name}");
                assert_eq!(texture.rgba.len(), 16, "{name}");
            }
            other => panic!("unexpected imported asset for {name}: {other:?}"),
        }
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_uses_extension_format_by_default() {
    let root = unique_temp_project_root("texture_import_extension_default");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("mismatched.png");
    fs::write(&png_path, tiny_jpeg_bytes()).unwrap();

    let error = importer_with_first_wave_plugin_fixtures()
        .import_from_source(
            &png_path,
            &AssetUri::parse("res://textures/mismatched.png").unwrap(),
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("decode image as `png` from extension"),
        "unexpected error: {error}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_can_guess_format_when_requested() {
    let root = unique_temp_project_root("texture_import_guess_format");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("mismatched.png");
    fs::write(&png_path, tiny_jpeg_bytes()).unwrap();
    let mut settings = toml::Table::new();
    settings.insert("image_format".to_string(), "guess".into());

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/mismatched.png").unwrap(),
            settings,
        )
        .unwrap();
    let imported = &outcome.root_entry().expect("root texture").asset;

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_can_use_explicit_source_format() {
    let root = unique_temp_project_root("texture_import_explicit_format");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("mismatched.png");
    fs::write(&png_path, tiny_jpeg_bytes()).unwrap();
    let mut settings = toml::Table::new();
    settings.insert("image_format".to_string(), "jpeg".into());

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/mismatched.png").unwrap(),
            settings,
        )
        .unwrap();
    let imported = &outcome.root_entry().expect("root texture").asset;

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_accepts_source_format_aliases() {
    let root = unique_temp_project_root("texture_import_source_format_alias");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("mismatched.png");
    fs::write(&png_path, tiny_rgb32f_image_bytes(ImageFormat::OpenExr)).unwrap();
    let mut settings = toml::Table::new();
    settings.insert("source_format".to_string(), "open_exr".into());

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/mismatched.png").unwrap(),
            settings,
        )
        .unwrap();
    let imported = &outcome.root_entry().expect("root texture").asset;

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_reports_actual_source_format_key() {
    let root = unique_temp_project_root("texture_import_source_format_diagnostic");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("checker.png");
    fs::write(&png_path, tiny_rgb_image_bytes(ImageFormat::Png)).unwrap();
    let mut settings = toml::Table::new();
    settings.insert("decode_format".to_string(), 1.into());

    let error = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/checker.png").unwrap(),
            settings,
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("image import setting `decode_format` must be a string"),
        "unexpected error: {error}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_applies_texture_import_settings_to_descriptor() {
    let root = unique_temp_project_root("texture_import_settings");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("height.png");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba([128, 128, 128, 255]))
        .save_with_format(&png_path, ImageFormat::Png)
        .unwrap();

    let mut sampler = toml::Table::new();
    sampler.insert("address_mode_u".to_string(), "repeat".into());
    sampler.insert("mag_filter".to_string(), "nearest".into());
    let mut settings = toml::Table::new();
    settings.insert("format".to_string(), "rgba16float".into());
    settings.insert("color_space".to_string(), "linear".into());
    settings.insert("dimension".to_string(), "3d".into());
    settings.insert(
        "usage".to_string(),
        toml::Value::Array(vec!["sampled".into(), "storage".into()]),
    );
    settings.insert(
        "asset_usage".to_string(),
        toml::Value::Array(vec!["render_world".into()]),
    );
    settings.insert("mip_count".to_string(), 3.into());
    settings.insert("depth_or_array_layers".to_string(), 4.into());
    settings.insert("sampler".to_string(), toml::Value::Table(sampler));

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/height.png").unwrap(),
            settings,
        )
        .unwrap();
    let imported = &outcome.root_entry().expect("root texture").asset;

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "rgba16float");
            assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
            assert_eq!(descriptor.dimension, RenderImageDimension::D3);
            assert_eq!(
                descriptor.usage,
                vec![RenderImageUsage::Sampled, RenderImageUsage::Storage]
            );
            assert_eq!(
                descriptor.asset_usage,
                vec![RenderImageAssetUsage::RenderWorld]
            );
            assert_eq!(descriptor.mip_count, 3);
            assert_eq!(descriptor.array_layer_count, 1);
            assert_eq!(descriptor.depth_or_array_layers, 4);
            assert_eq!(
                descriptor.sampler.address_mode_u,
                RenderSamplerAddressMode::Repeat
            );
            assert_eq!(descriptor.sampler.mag_filter, RenderSamplerFilter::Nearest);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_accepts_bevy_image_setting_aliases() {
    let root = unique_temp_project_root("texture_import_bevy_aliases");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("linear.png");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba([128, 128, 128, 255]))
        .save_with_format(&png_path, ImageFormat::Png)
        .unwrap();

    let mut settings = toml::Table::new();
    settings.insert("texture_format".to_string(), "rgba16float".into());
    settings.insert("is_srgb".to_string(), false.into());
    settings.insert("sampler".to_string(), "nearest".into());
    settings.insert("asset_usage".to_string(), "render_world".into());

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/linear.png").unwrap(),
            settings,
        )
        .unwrap();
    let imported = &outcome.root_entry().expect("root texture").asset;

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "rgba16float");
            assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
            assert_eq!(descriptor.sampler.mag_filter, RenderSamplerFilter::Nearest);
            assert_eq!(descriptor.sampler.min_filter, RenderSamplerFilter::Nearest);
            assert_eq!(
                descriptor.sampler.mipmap_filter,
                RenderSamplerFilter::Nearest
            );
            assert_eq!(
                descriptor.asset_usage,
                vec![RenderImageAssetUsage::RenderWorld]
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_normalizes_default_linear_rgba8_format() {
    let root = unique_temp_project_root("texture_import_linear_default");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("linear-default.png");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba([128, 128, 128, 255]))
        .save_with_format(&png_path, ImageFormat::Png)
        .unwrap();

    let mut settings = toml::Table::new();
    settings.insert("is_srgb".to_string(), false.into());

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/linear-default.png").unwrap(),
            settings,
        )
        .unwrap();
    let imported = &outcome.root_entry().expect("root texture").asset;

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, RGBA8_UNORM_FORMAT);
            assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_reinterprets_stacked_array_layout() {
    let root = unique_temp_project_root("texture_import_array_layout");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("stacked.png");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 4, |_x, y| {
        if y < 2 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 255, 255])
        }
    })
    .save_with_format(&png_path, ImageFormat::Png)
    .unwrap();

    for (layout_key, layout_value) in [("row_count", 2), ("row_height", 2)] {
        let mut array_layout = toml::Table::new();
        array_layout.insert(layout_key.to_string(), layout_value.into());
        let mut settings = toml::Table::new();
        settings.insert("array_layout".to_string(), toml::Value::Table(array_layout));

        let outcome = importer_with_first_wave_plugin_fixtures()
            .import_with_settings(
                &png_path,
                &AssetUri::parse("res://textures/stacked.png").unwrap(),
                settings,
            )
            .unwrap();
        let imported = &outcome.root_entry().expect("root texture").asset;

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 2, "{layout_key}");
                assert_eq!(texture.height, 2, "{layout_key}");
                assert_eq!(texture.rgba.len(), 2 * 4 * 4, "{layout_key}");
                let descriptor = texture.render_image_descriptor();
                assert_eq!(descriptor.height, 2, "{layout_key}");
                assert_eq!(descriptor.array_layer_count, 2, "{layout_key}");
                assert_eq!(descriptor.depth_or_array_layers, 2, "{layout_key}");
                assert_eq!(
                    descriptor.dimension,
                    RenderImageDimension::D2,
                    "{layout_key}"
                );
                assert_eq!(
                    texture
                        .upload_readiness(TextureUploadSupport::uncompressed_only())
                        .unsupported_reason(),
                    Some("rgba8 texture array/cubemap upload is not implemented"),
                    "{layout_key}"
                );
            }
            other => panic!("unexpected imported asset for {layout_key}: {other:?}"),
        }
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_texture_fixture_rejects_invalid_array_layout() {
    let root = unique_temp_project_root("texture_import_bad_array_layout");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("stacked.png");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 4, |_x, _y| Rgba([255, 255, 255, 255]))
        .save_with_format(&png_path, ImageFormat::Png)
        .unwrap();

    let mut array_layout = toml::Table::new();
    array_layout.insert("row_count".to_string(), 3.into());
    let mut settings = toml::Table::new();
    settings.insert("array_layout".to_string(), toml::Value::Table(array_layout));

    let error = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &png_path,
            &AssetUri::parse("res://textures/stacked.png").unwrap(),
            settings,
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("can not evenly divide height = 4 by layers = 3"),
        "unexpected error: {error}"
    );

    let _ = fs::remove_dir_all(root);
}

fn tiny_jpeg_bytes() -> Vec<u8> {
    tiny_rgb_image_bytes(ImageFormat::Jpeg)
}

fn tiny_rgb_image_bytes(format: ImageFormat) -> Vec<u8> {
    if matches!(format, ImageFormat::Hdr | ImageFormat::OpenExr) {
        return tiny_rgb32f_image_bytes(format);
    }

    let image = ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([255, 0, 0])
        } else {
            Rgb([0, 0, 255])
        }
    });
    let dynamic = DynamicImage::ImageRgb8(image);
    let mut bytes = std::io::Cursor::new(Vec::new());
    dynamic.write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

fn tiny_rgb32f_image_bytes(format: ImageFormat) -> Vec<u8> {
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
