use super::common::*;
use zircon_runtime::asset::ImportedAsset;
use zircon_runtime::core::framework::render::RenderImageDimension;

#[test]
fn ktx1_container_importer_rejects_truncated_key_value_metadata() {
    let mut bytes = tiny_ktx1_1d_bytes();
    bytes.truncate(64);
    write_u32(&mut bytes, 60, 16);

    let error = import_container_error("truncated-metadata.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata requires at least 80 bytes, got 64"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_reads_key_value_metadata_record() {
    let bytes = ktx1_with_key_value_metadata_bytes(&ktx1_key_value_metadata_record(b"author\0zr"));

    let imported = import_container_fixture("metadata.ktx", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 32);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.mip_count, 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx1_container_importer_reads_multiple_key_value_metadata_records() {
    let mut metadata = ktx1_key_value_metadata_record(b"author\0zr");
    metadata.extend_from_slice(&ktx1_key_value_metadata_record(b"tool\0texture-importer"));
    let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

    let imported = import_container_fixture("multi-metadata.ktx", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 32);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.dimension, RenderImageDimension::D1);
            assert_eq!(descriptor.mip_count, 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx1_container_importer_rejects_zero_key_value_metadata_record_size() {
    let metadata = 0_u32.to_le_bytes();
    let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

    let error = import_container_error("zero-metadata-record.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata record 0 keyAndValueByteSize must be nonzero"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_truncated_key_value_metadata_record_payload() {
    let mut metadata = Vec::new();
    metadata.extend_from_slice(&8_u32.to_le_bytes());
    metadata.extend_from_slice(&[1, 2, 3, 4]);
    let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

    let error = import_container_error("truncated-metadata-record.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata record 0 payload extends past declared ktx key/value metadata length"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_invalid_key_value_metadata_record_keys() {
    let cases: [(&str, Vec<u8>, &str); 4] = [
        (
            "unterminated-metadata-key.ktx",
            ktx1_key_value_metadata_record(b"author-value"),
            "ktx key/value metadata record 0 key must be NUL terminated",
        ),
        (
            "empty-metadata-key.ktx",
            ktx1_key_value_metadata_record(b"\0value"),
            "ktx key/value metadata record 0 key must be non-empty",
        ),
        (
            "bom-metadata-key.ktx",
            ktx1_key_value_metadata_record(&[0xef, 0xbb, 0xbf, b'k', 0, b'v']),
            "ktx key/value metadata record 0 key must be UTF-8 without BOM",
        ),
        (
            "invalid-utf8-metadata-key.ktx",
            ktx1_key_value_metadata_record(&[0xff, 0, b'v']),
            "ktx key/value metadata record 0 key must be UTF-8 without BOM",
        ),
    ];

    for (path, metadata, expected) in cases {
        let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx1_container_importer_rejects_invalid_later_key_value_metadata_record_key() {
    let mut metadata = ktx1_key_value_metadata_record(b"author\0zr");
    metadata.extend_from_slice(&ktx1_key_value_metadata_record(b"broken-key"));
    let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

    let error = import_container_error("invalid-second-metadata-key.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata record 1 key must be NUL terminated"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_nonzero_key_value_metadata_record_padding() {
    let mut metadata = ktx1_key_value_metadata_record(b"author\0zr");
    let last = metadata.len() - 1;
    metadata[last] = 1;
    let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

    let error = import_container_error("nonzero-metadata-padding.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata record 0 padding bytes must be zero"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_nonzero_later_key_value_metadata_record_padding() {
    let mut metadata = ktx1_key_value_metadata_record(b"author\0zr");
    let mut second_record = ktx1_key_value_metadata_record(b"tool\0texture-importer");
    let last = second_record.len() - 1;
    second_record[last] = 1;
    metadata.extend_from_slice(&second_record);
    let bytes = ktx1_with_key_value_metadata_bytes(&metadata);

    let error = import_container_error("nonzero-second-metadata-padding.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata record 1 padding bytes must be zero"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_unaligned_key_value_metadata() {
    let mut bytes = tiny_ktx1_1d_bytes();
    write_u32(&mut bytes, 60, 2);
    bytes.extend_from_slice(&[0, 0]);

    let error = import_container_error("unaligned-metadata.ktx", bytes);

    assert!(
        error.contains("ktx key/value metadata length must be a multiple of 4 bytes"),
        "unexpected error: {error}"
    );
}
