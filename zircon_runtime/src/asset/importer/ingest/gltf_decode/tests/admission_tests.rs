use std::collections::BTreeMap;
use std::io::Cursor;

use super::{decode_gltf_source, decode_gltf_source_with_buffer_limit};
use crate::asset::{AssetImportContext, AssetUri};

fn context(document: serde_json::Value) -> AssetImportContext {
    AssetImportContext::new(
        std::env::temp_dir().join("model.gltf"),
        AssetUri::parse("res://model.gltf").unwrap(),
        serde_json::to_vec(&document).unwrap(),
        toml::Table::new(),
    )
}

#[test]
fn gltf_declared_buffer_budget_rejects_fallback_and_repeated_sources_before_loading() {
    for buffers in [
        serde_json::json!([{
            "byteLength": 12,
            "extensions": {"EXT_meshopt_compression": {"fallback": true}}
        }]),
        serde_json::json!([
            {"byteLength": 8, "extensions": {"EXT_meshopt_compression": {"fallback": true}}},
            {"byteLength": 8, "extensions": {"EXT_meshopt_compression": {"fallback": true}}}
        ]),
        serde_json::json!([
            {"uri": "missing.bin", "byteLength": 8},
            {"uri": "missing.bin", "byteLength": 8}
        ]),
    ] {
        let input = context(serde_json::json!({"asset": {"version": "2.0"}, "buffers": buffers}));
        let error = decode_gltf_source_with_buffer_limit(&input, 8).unwrap_err();
        assert!(
            error.to_string().contains("decoded buffer budget"),
            "{error}"
        );
    }
}

#[test]
fn gltf_buffer_budget_charges_actual_padded_payload_for_each_retained_buffer() {
    let mut input = context(serde_json::json!({
        "asset": {"version": "2.0"},
        "buffers": [
            {"uri": "mesh.bin", "byteLength": 1},
            {"uri": "mesh.bin", "byteLength": 1}
        ]
    }));
    let path = input.source_path.parent().unwrap().join("mesh.bin");
    input = input.with_source_file_snapshots(BTreeMap::from([(path, vec![7; 5])]));

    let error = decode_gltf_source_with_buffer_limit(&input, 12).unwrap_err();
    assert!(
        error.to_string().contains("decoded buffer budget"),
        "{error}"
    );
    let decoded = decode_gltf_source_with_buffer_limit(&input, 16).unwrap();
    assert_eq!(decoded.buffers.len(), 2);
    assert_eq!(decoded.buffers[0].0, [7, 7, 7, 7, 7, 0, 0, 0]);
    assert_eq!(decoded.buffers[0].0, decoded.buffers[1].0);
}

#[test]
fn gltf_embedded_buffer_budget_accepts_exact_limit_and_rejects_next_buffer() {
    let input = context(serde_json::json!({
        "asset": {"version": "2.0"},
        "buffers": [
            {"uri": "data:application/octet-stream;base64,AQIDBA==", "byteLength": 4},
            {"uri": "data:application/octet-stream;base64,BQYHCA==", "byteLength": 4}
        ]
    }));
    let decoded = decode_gltf_source_with_buffer_limit(&input, 8).unwrap();
    assert_eq!(decoded.buffers[0].0, [1, 2, 3, 4]);
    assert_eq!(decoded.buffers[1].0, [5, 6, 7, 8]);
    let error = decode_gltf_source_with_buffer_limit(&input, 7).unwrap_err();
    assert!(
        error.to_string().contains("decoded buffer budget"),
        "{error}"
    );
}

#[test]
fn gltf_embedded_buffer_budget_checks_actual_payload_before_decoding() {
    let input = context(serde_json::json!({
        "asset": {"version": "2.0"},
        "buffers": [{"uri": "data:application/octet-stream;base64,AQIDBAUGBwg=", "byteLength": 1}]
    }));
    let error = decode_gltf_source_with_buffer_limit(&input, 4).unwrap_err();
    assert!(
        error.to_string().contains("decoded buffer budget"),
        "{error}"
    );
    assert_eq!(
        decode_gltf_source_with_buffer_limit(&input, 8)
            .unwrap()
            .buffers[0]
            .0
            .len(),
        8
    );
}

#[test]
fn gltf_data_uri_decoder_preserves_optional_base64_padding_and_rejects_invalid_payloads() {
    for uri in [
        "data:application/octet-stream;base64,AQIDBA==",
        "data:application/octet-stream;base64,AQIDBA",
        "data:AQIDBA==",
    ] {
        assert_eq!(
            super::buffers::decode_data_uri(uri, 4).unwrap(),
            [1, 2, 3, 4]
        );
    }
    for payload in ["A", "A===", "!!!!", "AQ=I", "AQIDBA==="] {
        let uri = format!("data:application/octet-stream;base64,{payload}");
        assert!(
            super::buffers::decode_data_uri(&uri, 16).is_err(),
            "{payload}"
        );
    }
}

#[test]
fn gltf_image_budget_charges_repeated_png_and_webp_pixels() {
    for format in [image::ImageFormat::Png, image::ImageFormat::WebP] {
        let mut encoded = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([17, 41, 93, 128]),
        ))
        .write_to(&mut encoded, format)
        .unwrap();
        let mut input = context(serde_json::json!({
            "asset": {"version": "2.0"},
            "images": [{"uri": "pixel"}, {"uri": "pixel"}]
        }));
        let path = input.source_path.parent().unwrap().join("pixel");
        input = input.with_source_file_snapshots(BTreeMap::from([(path, encoded.into_inner())]));
        let error = decode_gltf_source_with_buffer_limit(&input, 7).unwrap_err();
        assert!(
            error.to_string().contains("decoded image budget"),
            "{format:?}: {error}"
        );
        let decoded = decode_gltf_source_with_buffer_limit(&input, 8).unwrap();
        assert_eq!(decoded.images[0].pixels, [17, 41, 93, 128]);
        assert_eq!(decoded.images[0].pixels, decoded.images[1].pixels);
    }
}

#[test]
fn gltf_image_views_share_the_retained_buffer_budget() {
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(1, 1)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let encoded = encoded.into_inner();
    let length = encoded.len();
    let padded_length = length.next_multiple_of(4) as u64;
    let mut input = context(serde_json::json!({
        "asset": {"version": "2.0"},
        "buffers": [{"uri": "image.bin", "byteLength": length}],
        "bufferViews": [{"buffer": 0, "byteLength": length}],
        "images": [{"bufferView": 0, "mimeType": "image/png"}]
    }));
    let path = input.source_path.parent().unwrap().join("image.bin");
    input = input.with_source_file_snapshots(BTreeMap::from([(path, encoded)]));
    let error = decode_gltf_source_with_buffer_limit(&input, padded_length + 3).unwrap_err();
    assert!(
        error.to_string().contains("decoded image budget"),
        "{error}"
    );
    let decoded = decode_gltf_source_with_buffer_limit(&input, padded_length + 4).unwrap();
    assert_eq!(decoded.images[0].pixels.len(), 4);
}

#[test]
fn gltf_image_data_uri_reserves_encoded_temporary_storage_before_pixels() {
    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg==";
    let input = context(serde_json::json!({
        "asset": {"version": "2.0"},
        "images": [{"uri": format!("data:image/png;base64,{PNG}")}]
    }));
    let error = decode_gltf_source_with_buffer_limit(&input, 4).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("budget exceeded by embedded data URI"),
        "{error}"
    );
    let decoded = decode_gltf_source_with_buffer_limit(&input, 76).unwrap();
    assert_eq!(decoded.images[0].pixels, [255; 4]);
}

#[test]
fn gltf_image_decoder_preserves_native_endian_sixteen_bit_samples() {
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::ImageLuma16(image::ImageBuffer::from_pixel(1, 1, image::Luma([0x1234])))
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let decoded = super::images::decode_external_image(
        encoded.get_ref(),
        std::path::Path::new(""),
        Some("IMAGE/PNG"),
        &mut super::budget::DecodedBudget::new(2),
    )
    .unwrap();
    assert_eq!(decoded.format, gltf::image::Format::R16);
    assert_eq!(decoded.pixels, 0x1234_u16.to_ne_bytes());
}

#[test]
fn gltf_opaque_webp_conversion_admits_both_live_pixel_buffers() {
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(1, 1, image::Rgb([17, 41, 93])))
        .write_to(&mut encoded, image::ImageFormat::WebP)
        .unwrap();
    let error = super::images::decode_external_image(
        encoded.get_ref(),
        std::path::Path::new("pixel.webp"),
        None,
        &mut super::budget::DecodedBudget::new(6),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("decoded image budget"),
        "{error}"
    );
    let decoded = super::images::decode_external_image(
        encoded.get_ref(),
        std::path::Path::new("pixel.webp"),
        None,
        &mut super::budget::DecodedBudget::new(7),
    )
    .unwrap();
    assert_eq!(decoded.pixels, [17, 41, 93, 255]);
}

#[test]
fn gltf_direct_source_cache_keeps_the_first_generation_after_file_removal() {
    let root = super::plugins07_deferred_buffer_diagnostic_tests::auxiliary_test_root("gltf-cache");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("mesh.bin");
    std::fs::write(&path, [1, 2, 3, 4]).unwrap();
    let mut input = context(serde_json::json!({"asset": {"version": "2.0"}}));
    input.source_path = root.join("model.gltf");
    let mut sources = super::sources::ExternalSources::new(&input);
    assert_eq!(sources.read("mesh.bin", 4).unwrap().1, [1, 2, 3, 4]);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(sources.read("./mesh.bin", 4).unwrap().1, [1, 2, 3, 4]);
    #[cfg(windows)]
    assert_eq!(sources.read("MESH.BIN", 4).unwrap().1, [1, 2, 3, 4]);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gltf_external_buffer_budget_rejects_actual_size_during_read_admission() {
    let root =
        super::plugins07_deferred_buffer_diagnostic_tests::auxiliary_test_root("gltf-read-cap");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("mesh.bin"), [7; 8]).unwrap();
    let mut input = context(serde_json::json!({
        "asset": {"version": "2.0"},
        "buffers": [{"uri": "mesh.bin", "byteLength": 1}]
    }));
    input.source_path = root.join("model.gltf");
    let error = decode_gltf_source_with_buffer_limit(&input, 4).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("8 bytes, exceeding the 4-byte limit"),
        "{error}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(windows)]
fn gltf_source_snapshot_charges_windows_case_aliases_once() {
    let root = super::plugins07_deferred_buffer_diagnostic_tests::auxiliary_test_root("gltf-alias");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("mesh.bin"), [1, 2, 3, 4]).unwrap();
    let document = br#"{"asset":{"version":"2.0"},"buffers":[{"uri":"mesh.bin","byteLength":4},{"uri":"MESH.BIN","byteLength":4}]}"#;
    let (snapshots, _) = super::snapshot_external_gltf_sources(
        &root,
        &root.join("model.gltf"),
        &AssetUri::parse("res://model.gltf").unwrap(),
        document,
        &BTreeMap::new(),
        4,
    )
    .unwrap();
    assert_eq!(snapshots.len(), 1);
    std::fs::remove_file(root.join("mesh.bin")).unwrap();
    let (additional, _) = super::snapshot_external_gltf_sources(
        &root,
        &root.join("model.gltf"),
        &AssetUri::parse("res://model.gltf").unwrap(),
        document,
        &snapshots,
        0,
    )
    .unwrap();
    assert!(additional.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gltf_every_image_view_returns_a_parse_error_for_an_invalid_byte_range() {
    for mime_type in ["image/png", "image/jpeg", "image/webp"] {
        for offset in [4_u64, u64::MAX - 3] {
            let input = context(serde_json::json!({
                "asset": {"version": "2.0"},
                "buffers": [{"uri": "data:application/octet-stream;base64,AQIDBA==", "byteLength": 4}],
                "bufferViews": [{"buffer": 0, "byteOffset": offset, "byteLength": 8}],
                "images": [{"bufferView": 0, "mimeType": mime_type}]
            }));
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                decode_gltf_source(&input)
            }));
            let error = result
                .expect("untrusted image ranges must not panic")
                .unwrap_err();
            assert!(error.to_string().contains("range"), "{mime_type}: {error}");
        }
    }
}
