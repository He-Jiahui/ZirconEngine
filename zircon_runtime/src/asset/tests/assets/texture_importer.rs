use std::fs;

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, Rgba};

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::importer_with_first_wave_plugin_fixtures;
use crate::asset::{
    AssetUri, ImportedAsset, TextureAsset, TextureAssetDescriptor, TextureUploadCompressionFamily,
    TextureUploadReadiness, TextureUploadSupport,
};
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

#[test]
fn texture_upload_readiness_reports_compressed_container_support() {
    let uri = AssetUri::parse("res://textures/bc1.dds").unwrap();
    let mut bytes = vec![0_u8; 128];
    bytes.extend_from_slice(&[1_u8; 8]);
    let texture = TextureAsset::new_container(uri, 4, 4, "dds/DXT1", bytes, 1, 1);

    let unsupported = texture.upload_readiness(TextureUploadSupport::uncompressed_only());
    assert_eq!(
        unsupported.unsupported_reason(),
        Some("gpu device does not support BC compressed textures")
    );
    let short = TextureAsset::new_container(
        AssetUri::parse("res://textures/short-bc1.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        vec![0_u8; 129],
        1,
        1,
    );
    assert_eq!(
        short
            .upload_readiness(TextureUploadSupport {
                bc: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("container texture payload format dds/dxt1 has 1 image bytes but needs at least 8")
    );

    match texture.upload_readiness(TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    }) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
            assert_eq!(plan.data_offset, 128);
            assert_eq!(plan.data_length, None);
            assert_eq!(plan.bytes_per_block, 8);
        }
        other => panic!("expected BC upload-ready texture, got {other:?}"),
    }

    for (fourcc, expected_bytes_per_block) in [
        ("ATI1", 8),
        ("BC4U", 8),
        ("BC4S", 8),
        ("ATI2", 16),
        ("BC5U", 16),
        ("BC5S", 16),
    ] {
        let mut bytes = vec![0_u8; 128];
        bytes.extend(vec![1_u8; expected_bytes_per_block as usize]);
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/{fourcc}.dds")).unwrap(),
            4,
            4,
            format!("dds/{fourcc}"),
            bytes,
            1,
            1,
        );
        match texture.upload_readiness(TextureUploadSupport {
            bc: true,
            ..TextureUploadSupport::uncompressed_only()
        }) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(plan.format, format!("dds/{}", fourcc.to_ascii_lowercase()));
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 128);
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!("expected DDS {fourcc} upload-ready texture, got {other:?}"),
        }
    }

    for (dxgi, expected_bytes_per_block) in
        [(80, 8), (81, 8), (83, 16), (84, 16), (95, 16), (96, 16)]
    {
        let mut bytes = vec![0_u8; 148];
        bytes.extend(vec![1_u8; expected_bytes_per_block as usize]);
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/dxgi-{dxgi}.dds")).unwrap(),
            4,
            4,
            format!("dds/dxgi-{dxgi}"),
            bytes,
            1,
            1,
        );
        match texture.upload_readiness(TextureUploadSupport {
            bc: true,
            ..TextureUploadSupport::uncompressed_only()
        }) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(plan.format, format!("dds/dxgi-{dxgi}"));
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 148);
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!("expected DXGI {dxgi} BC upload-ready texture, got {other:?}"),
        }
    }
}

#[test]
fn texture_upload_readiness_extracts_ktx_level_payload_offsets() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/ktx-bc1.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        ktx1_bc1_level_bytes(),
        1,
        1,
    );

    match ktx1.upload_readiness(support) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.format, "ktx/gl-internal-0x000083f1");
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
            assert_eq!(plan.data_offset, 68);
            assert_eq!(plan.data_length, Some(8));
            assert_eq!(plan.bytes_per_block, 8);
        }
        other => panic!("expected KTX1 BC upload-ready texture, got {other:?}"),
    }

    for (gl_internal_format, expected_bytes_per_block, expected_level_bytes) in [
        (0x8dbb, 8, 8),
        (0x8dbc, 8, 8),
        (0x8dbd, 16, 16),
        (0x8dbe, 16, 16),
        (0x8e8c, 16, 16),
        (0x8e8d, 16, 16),
        (0x8e8e, 16, 16),
        (0x8e8f, 16, 16),
    ] {
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/ktx-bc-{gl_internal_format:x}.ktx")).unwrap(),
            4,
            4,
            format!("ktx/gl-internal-0x{gl_internal_format:08x}"),
            ktx1_compressed_level_bytes(gl_internal_format, expected_level_bytes),
            1,
            1,
        );
        match texture.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(
                    plan.format,
                    format!("ktx/gl-internal-0x{gl_internal_format:08x}")
                );
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 68);
                assert_eq!(plan.data_length, Some(expected_level_bytes));
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!(
                "expected KTX1 BC gl-internal 0x{gl_internal_format:08x} upload-ready texture, got {other:?}"
            ),
        }
    }

    let ktx1_astc = TextureAsset::new_container(
        AssetUri::parse("res://textures/ktx-astc.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000093b0",
        ktx1_compressed_level_bytes(0x93b0, 16),
        1,
        1,
    );
    assert_eq!(
        ktx1_astc.upload_readiness(support).unsupported_reason(),
        Some("gpu device does not support ASTC compressed textures")
    );

    match ktx1_astc.upload_readiness(TextureUploadSupport {
        bc: true,
        astc_ldr: true,
        ..TextureUploadSupport::uncompressed_only()
    }) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.format, "ktx/gl-internal-0x000093b0");
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Astc);
            assert_eq!(plan.data_offset, 68);
            assert_eq!(plan.data_length, Some(16));
            assert_eq!(plan.block_width, 4);
            assert_eq!(plan.block_height, 4);
            assert_eq!(plan.bytes_per_block, 16);
        }
        other => panic!("expected KTX1 ASTC upload-ready texture, got {other:?}"),
    }

    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/ktx2-bc1.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        ktx2_bc1_level_bytes(),
        1,
        1,
    );

    match ktx2.upload_readiness(support) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.format, "ktx2/vk-133/supercompression-0");
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
            assert_eq!(plan.data_offset, 104);
            assert_eq!(plan.data_length, Some(8));
            assert_eq!(plan.bytes_per_block, 8);
        }
        other => panic!("expected KTX2 BC upload-ready texture, got {other:?}"),
    }

    for (vk_format, expected_bytes_per_block, expected_level_bytes) in [
        (139, 8, 8),
        (140, 8, 8),
        (141, 16, 16),
        (142, 16, 16),
        (143, 16, 16),
        (144, 16, 16),
    ] {
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/ktx2-bc-{vk_format}.ktx2")).unwrap(),
            4,
            4,
            format!("ktx2/vk-{vk_format}/supercompression-0"),
            ktx2_compressed_level_bytes(vk_format, expected_level_bytes),
            1,
            1,
        );
        match texture.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(
                    plan.format,
                    format!("ktx2/vk-{vk_format}/supercompression-0")
                );
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 104);
                assert_eq!(plan.data_length, Some(expected_level_bytes));
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!("expected KTX2 BC vk-{vk_format} upload-ready texture, got {other:?}"),
        }
    }
}

#[test]
fn texture_upload_readiness_rejects_compressed_mips_and_arrays_until_full_upload_exists() {
    let mut bytes = vec![0_u8; 128];
    bytes.extend_from_slice(&[1_u8; 8]);
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mip_chain = TextureAsset::new_container(
        AssetUri::parse("res://textures/bc1-mips.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        bytes.clone(),
        2,
        1,
    );
    assert_eq!(
        mip_chain.upload_readiness(support).unsupported_reason(),
        Some("compressed texture mip-chain upload is not implemented")
    );

    let array_layers = TextureAsset::new_container(
        AssetUri::parse("res://textures/bc1-array.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        bytes,
        1,
        6,
    );
    assert_eq!(
        array_layers.upload_readiness(support).unsupported_reason(),
        Some("compressed texture array/cubemap upload is not implemented")
    );
}

#[test]
fn texture_upload_readiness_rejects_compressed_1d_and_etc2_3d_boundaries() {
    let support = TextureUploadSupport {
        bc: true,
        etc2: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let mut d1_descriptor = TextureAssetDescriptor::container("ktx/gl-internal-0x000083f1", 1, 1);
    d1_descriptor.dimension = RenderImageDimension::D1;
    let d1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/line-bc1.ktx").unwrap(),
        4,
        1,
        "ktx/gl-internal-0x000083f1",
        ktx1_bc1_level_bytes(),
        1,
        1,
    )
    .with_descriptor(d1_descriptor);
    assert_eq!(
        d1.upload_readiness(support).unsupported_reason(),
        Some("compressed texture 1d upload is not implemented")
    );

    let mut etc2_3d_descriptor =
        TextureAssetDescriptor::container("ktx2/vk-147/supercompression-0", 1, 1);
    etc2_3d_descriptor.dimension = RenderImageDimension::D3;
    etc2_3d_descriptor.depth_or_array_layers = 4;
    etc2_3d_descriptor.array_layer_count = 1;
    let etc2_3d = TextureAsset::new_container(
        AssetUri::parse("res://textures/volume-etc2.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-147/supercompression-0",
        ktx2_etc2_level_bytes(),
        1,
        1,
    )
    .with_descriptor(etc2_3d_descriptor);
    assert_eq!(
        etc2_3d.upload_readiness(support).unsupported_reason(),
        Some("compressed texture ETC2 3d upload is not implemented")
    );
}

#[test]
fn texture_upload_readiness_rejects_short_ktx_level_declarations() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let mut short_ktx1 = ktx1_bc1_level_bytes();
    write_u32_le(&mut short_ktx1, 64, 1);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/short-level.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        short_ktx1,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("container texture payload format ktx/gl-internal-0x000083f1 declares 1 image bytes but needs at least 8")
    );

    let mut truncated_ktx2 = ktx2_bc1_level_bytes();
    write_u64_le(&mut truncated_ktx2, 88, 16);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/truncated-level.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        truncated_ktx2,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("container texture payload format ktx2/vk-133/supercompression-0 declares 16 image bytes but only 8 are available")
    );
}

#[test]
fn texture_upload_readiness_rejects_malformed_ktx_headers_before_level_parsing() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let mut bad_ktx1_magic = ktx1_bc1_level_bytes();
    bad_ktx1_magic[0] = 0;
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-magic.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_magic,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx1_endian = ktx1_bc1_level_bytes();
    write_u32_le(&mut bad_ktx1_endian, 12, 0);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-endian.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_endian,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx2_magic = ktx2_bc1_level_bytes();
    bad_ktx2_magic[0] = 0;
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-magic.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        bad_ktx2_magic,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_rejects_malformed_ktx2_level_index_entries() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut incomplete_index = ktx2_bc1_level_bytes();
    incomplete_index.truncate(96);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/incomplete-index.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        incomplete_index,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut mismatched_uncompressed_len = ktx2_bc1_level_bytes();
    write_u64_le(&mut mismatched_uncompressed_len, 96, 16);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-index.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_uncompressed_len,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_rejects_ktx_descriptor_header_format_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut mismatched_ktx1_format = ktx1_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx1_format, 28, 0x9274);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-format.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        mismatched_ktx1_format,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut mismatched_ktx2_format = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_format, 12, 147);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-format.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_format,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut mismatched_ktx2_supercompression = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_supercompression, 44, 1);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-supercompression.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_supercompression,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let missing_supercompression_token = TextureAsset::new_container(
        AssetUri::parse("res://textures/missing-supercompression.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133",
        ktx2_bc1_level_bytes(),
        1,
        1,
    );
    assert_eq!(
        missing_supercompression_token
            .upload_readiness(support)
            .unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_reports_supercompression_and_astc_3d_boundaries() {
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/super.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-37/supercompression-1",
        vec![0; 96],
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(TextureUploadSupport::all_compressed())
            .unsupported_reason(),
        Some("ktx2 supercompression 1 requires a transcoding backend")
    );

    let mut astc_bytes = vec![0_u8; 16];
    astc_bytes.extend_from_slice(&[1_u8; 16]);
    let astc_3d = TextureAsset::new_container(
        AssetUri::parse("res://textures/volume.astc").unwrap(),
        4,
        4,
        "astc/4x4x4",
        astc_bytes,
        1,
        1,
    );
    assert_eq!(
        astc_3d
            .upload_readiness(TextureUploadSupport {
                astc_ldr: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("gpu device does not support ASTC sliced 3d textures")
    );

    let mut astc_3d_bytes = vec![0_u8; 16];
    astc_3d_bytes.extend_from_slice(&[1_u8; 16]);
    let astc_3d = TextureAsset::new_container(
        AssetUri::parse("res://textures/volume-3x3x3.astc").unwrap(),
        3,
        3,
        "astc/3x3x3",
        astc_3d_bytes,
        1,
        1,
    );
    assert_eq!(
        astc_3d
            .upload_readiness(TextureUploadSupport {
                astc_ldr: true,
                astc_sliced_3d: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("astc 3d block payload upload is not implemented")
    );

    let astc_unknown = TextureAsset::new_container(
        AssetUri::parse("res://textures/unknown-block.astc").unwrap(),
        7,
        7,
        "astc/7x7x7",
        vec![0_u8; 32],
        1,
        1,
    );
    assert_eq!(
        astc_unknown
            .upload_readiness(TextureUploadSupport::all_compressed())
            .unsupported_reason(),
        Some("texture container format astc/7x7x7 is not upload-ready")
    );
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

fn ktx1_bc1_level_bytes() -> Vec<u8> {
    ktx1_compressed_level_bytes(0x83f1, 8)
}

fn ktx1_compressed_level_bytes(gl_internal_format: u32, level_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 64];
    bytes[0..12].copy_from_slice(b"\xABKTX 11\xBB\r\n\x1A\n");
    write_u32_le(&mut bytes, 12, 0x0403_0201);
    write_u32_le(&mut bytes, 28, gl_internal_format);
    write_u32_le(&mut bytes, 36, 4);
    write_u32_le(&mut bytes, 40, 4);
    write_u32_le(&mut bytes, 56, 1);
    write_u32_le(&mut bytes, 60, 0);
    bytes.extend_from_slice(&(level_bytes as u32).to_le_bytes());
    bytes.extend(vec![1_u8; level_bytes]);
    bytes
}

fn ktx2_bc1_level_bytes() -> Vec<u8> {
    ktx2_compressed_level_bytes(133, 8)
}

fn ktx2_etc2_level_bytes() -> Vec<u8> {
    ktx2_compressed_level_bytes(147, 32)
}

fn ktx2_compressed_level_bytes(vk_format: u32, level_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 104];
    bytes[0..12].copy_from_slice(b"\xABKTX 20\xBB\r\n\x1A\n");
    write_u32_le(&mut bytes, 12, vk_format);
    write_u32_le(&mut bytes, 16, 1);
    write_u32_le(&mut bytes, 20, 4);
    write_u32_le(&mut bytes, 24, 4);
    write_u32_le(&mut bytes, 40, 1);
    write_u32_le(&mut bytes, 44, 0);
    write_u64_le(&mut bytes, 80, 104);
    write_u64_le(&mut bytes, 88, level_bytes as u64);
    write_u64_le(&mut bytes, 96, level_bytes as u64);
    bytes.extend(vec![1_u8; level_bytes]);
    bytes
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
