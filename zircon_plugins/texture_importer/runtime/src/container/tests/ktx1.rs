use super::common::*;
use zircon_runtime::asset::{ImportedAsset, TexturePayload};
use zircon_runtime::core::framework::render::RenderImageDimension;

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
fn ktx1_container_importer_rejects_invalid_type_format_header_fields() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 6] = [
        (
            "zero-type-size.ktx",
            |bytes| write_u32(bytes, 20, 0),
            "ktx glTypeSize must be nonzero",
        ),
        (
            "unsupported-type-size.ktx",
            |bytes| write_u32(bytes, 20, 3),
            "ktx glTypeSize must be 1, 2, or 4 bytes",
        ),
        (
            "format-without-type.ktx",
            |bytes| write_u32(bytes, 16, 0),
            "ktx glType and glFormat must both be zero for compressed data or both be nonzero for uncompressed data",
        ),
        (
            "type-without-format.ktx",
            |bytes| write_u32(bytes, 24, 0),
            "ktx glType and glFormat must both be zero for compressed data or both be nonzero for uncompressed data",
        ),
        (
            "zero-internal-format.ktx",
            |bytes| write_u32(bytes, 28, 0),
            "ktx glInternalFormat must be nonzero",
        ),
        (
            "unsized-internal-format.ktx",
            |bytes| write_u32(bytes, 28, 0x1908),
            "ktx glInternalFormat must not equal glFormat",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx1_1d_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx1_container_importer_rejects_invalid_base_format_and_compressed_type_size() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 2] = [
        (
            "zero-base-internal-format.ktx",
            |bytes| write_u32(bytes, 32, 0),
            "ktx glBaseInternalFormat must be nonzero",
        ),
        (
            "compressed-type-size-not-one.ktx",
            |bytes| {
                write_u32(bytes, 16, 0);
                write_u32(bytes, 20, 2);
                write_u32(bytes, 24, 0);
            },
            "ktx glTypeSize must be 1 for compressed data",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx1_1d_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
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
fn ktx1_container_importer_rejects_nonzero_intermediate_mip_padding() {
    let mut bytes = ktx1_two_mip_prefix_bytes(5);
    let first_level_padding_offset = 64 + 4 + 5;
    bytes[first_level_padding_offset] = 1;
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2, 3, 4]);

    let error = import_container_error("nonzero-level-padding.ktx", bytes);

    assert!(
        error.contains("ktx first mip level payload padding bytes must be zero"),
        "unexpected error: {error}"
    );
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
fn ktx1_container_importer_reads_3d_dimension() {
    let mut bytes = ktx1_layer_face_bytes(0, 1);
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
fn ktx1_container_importer_rejects_3d_array_header() {
    let mut bytes = ktx1_layer_face_bytes(2, 1);
    write_u32(&mut bytes, 40, 8);
    write_u32(&mut bytes, 44, 5);

    let error = import_container_error("volume-array.ktx", bytes);

    assert!(
        error.contains("ktx 3d textures must not declare array layers"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_1d_cubemap_header() {
    let bytes = ktx1_layer_face_bytes(0, 6);

    let error = import_container_error("strip-cubemap.ktx", bytes);

    assert!(
        error.contains("ktx cubemap faces must be 2d and square, got 32x0"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_non_square_cubemap_header() {
    let mut bytes = ktx1_layer_face_bytes(0, 6);
    write_u32(&mut bytes, 40, 16);

    let error = import_container_error("non-square-cubemap.ktx", bytes);

    assert!(
        error.contains("ktx cubemap faces must be 2d and square, got 32x16"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx1_container_importer_rejects_3d_cubemap_header() {
    let mut bytes = ktx1_layer_face_bytes(0, 6);
    write_u32(&mut bytes, 40, 8);
    write_u32(&mut bytes, 44, 5);

    let error = import_container_error("volume-cubemap.ktx", bytes);

    assert!(
        error.contains("ktx cubemap textures must not declare 3d depth"),
        "unexpected error: {error}"
    );
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
