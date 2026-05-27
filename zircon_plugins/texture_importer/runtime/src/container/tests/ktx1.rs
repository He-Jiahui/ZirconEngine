use super::super::*;
use super::common::*;

#[test]
fn ktx1_container_importer_reads_1d_dimension() {
    let imported = import_container_fixture("strip.ktx", tiny_ktx1_1d_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 32);
            assert_eq!(texture.height, 1);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.dimension, RenderImageDimension::D1);
            assert_eq!(descriptor.depth_or_array_layers, 1);
            assert_eq!(descriptor.array_layer_count, 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

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
fn ktx1_container_importer_rejects_truncated_first_level_image_size() {
    let mut bytes = tiny_ktx1_1d_bytes();
    bytes.truncate(64);

    let error = import_container_error("truncated-level-size.ktx", bytes);

    assert!(
        error.contains("ktx first mip level imageSize requires at least 68 bytes, got 64"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_truncated_first_level_payload() {
    let mut bytes = tiny_ktx1_1d_bytes();
    write_u32(&mut bytes, 64, 8);
    bytes.extend_from_slice(&[1, 2, 3, 4]);

    let error = import_container_error("truncated-level-payload.ktx", bytes);

    assert!(
        error.contains("ktx first mip level payload requires at least 76 bytes, got 72"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_truncated_second_level_image_size() {
    let bytes = ktx1_two_mip_prefix_bytes(5);

    let error = import_container_error("truncated-second-level-size.ktx", bytes);

    assert!(
        error.contains("ktx mip level 1 imageSize requires at least 80 bytes, got 76"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_truncated_second_level_payload() {
    let mut bytes = ktx1_two_mip_prefix_bytes(5);
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2]);

    let error = import_container_error("truncated-second-level-payload.ktx", bytes);

    assert!(
        error.contains("ktx mip level 1 payload requires at least 84 bytes, got 82"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_reads_complete_mip_chain() {
    let mut bytes = ktx1_two_mip_prefix_bytes(5);
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2, 3, 4]);
    let imported = import_container_fixture("mipped.ktx", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.mip_count, 2);
            match texture.payload {
                TexturePayload::Container { mip_count, .. } => {
                    assert_eq!(mip_count, 2);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx1_container_importer_rejects_mip_count_larger_than_extent_chain() {
    let mut bytes = tiny_ktx1_1d_bytes();
    write_u32(&mut bytes, 36, 8);
    write_u32(&mut bytes, 56, 5);

    let error = import_container_error("overlarge-mip-count.ktx", bytes);

    assert!(
        error.contains("ktx mip level count 5 exceeds maximum 4 for extent 8x1x1"),
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

#[test]
fn ktx1_3d_container_keeps_depth_separate_from_array_layers() {
    let mut bytes = ktx1_layer_face_bytes(2, 6);
    write_u32(&mut bytes, 40, 8);
    write_u32(&mut bytes, 44, 5);
    let imported = import_container_fixture("volume.ktx", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.dimension, RenderImageDimension::D3);
            assert_eq!(descriptor.depth_or_array_layers, 5);
            assert_eq!(descriptor.array_layer_count, 1);
            match texture.payload {
                TexturePayload::Container { array_layers, .. } => {
                    assert_eq!(array_layers, 1);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx1_container_importer_rejects_3d_texture_without_height() {
    let mut bytes = tiny_ktx1_1d_bytes();
    write_u32(&mut bytes, 44, 5);

    let error = import_container_error("volume-without-height.ktx", bytes);

    assert!(
        error.contains("ktx 3d texture height must be nonzero when depth is nonzero"),
        "unexpected error: {error}"
    );
}
