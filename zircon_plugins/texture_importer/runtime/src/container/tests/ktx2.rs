use super::super::*;
use super::common::*;

#[test]
fn ktx2_container_importer_reads_layers_faces_and_mips() {
    let imported = import_container_fixture("array.ktx2", tiny_ktx2_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.height, 8);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.dimension, RenderImageDimension::D2);
            assert_eq!(descriptor.depth_or_array_layers, 12);
            assert_eq!(descriptor.mip_count, 4);
            assert_eq!(descriptor.array_layer_count, 12);
            match texture.payload {
                TexturePayload::Container {
                    format,
                    mip_count,
                    array_layers,
                    ..
                } => {
                    assert_eq!(format, "ktx2/vk-37/supercompression-1");
                    assert_eq!(mip_count, 4);
                    assert_eq!(array_layers, 12);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx2_container_importer_accepts_known_supercompression_schemes() {
    let cases = [
        (KTX2_SUPERCOMPRESSION_NONE, "ktx2/vk-37/supercompression-0"),
        (
            KTX2_SUPERCOMPRESSION_BASIS_LZ,
            "ktx2/vk-37/supercompression-1",
        ),
        (
            KTX2_SUPERCOMPRESSION_ZSTANDARD,
            "ktx2/vk-37/supercompression-2",
        ),
        (KTX2_SUPERCOMPRESSION_ZLIB, "ktx2/vk-37/supercompression-3"),
    ];

    for (scheme, expected_format) in cases {
        let mut bytes = tiny_ktx2_bytes();
        write_u32(&mut bytes, 44, scheme);

        let imported = import_container_fixture("supercompression.ktx2", bytes);

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(
                    texture.render_image_descriptor().format,
                    expected_format,
                    "unexpected descriptor format for scheme {scheme}"
                );
                match texture.payload {
                    TexturePayload::Container { format, .. } => {
                        assert_eq!(format, expected_format);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }
}

#[test]
fn ktx2_container_importer_rejects_unsupported_supercompression_schemes() {
    let cases = [4, 0x0001_0000, 0xffff_ffff];

    for scheme in cases {
        let mut bytes = tiny_ktx2_bytes();
        write_u32(&mut bytes, 44, scheme);

        let error = import_container_error("unsupported-supercompression.ktx2", bytes);

        assert!(
            error.contains(&format!(
                "ktx2 supercompression scheme {scheme} is not supported by container importer"
            )),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_zero_type_size() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 16, 0);

    let error = import_container_error("zero-type-size.ktx2", bytes);

    assert!(
        error.contains("ktx2 type size must be nonzero"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_mip_count_larger_than_extent_chain() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 20, 8);
    write_u32(&mut bytes, 24, 4);
    write_u32(&mut bytes, 40, 5);

    let error = import_container_error("overlarge-mip-count.ktx2", bytes);

    assert!(
        error.contains("ktx2 mip level count 5 exceeds maximum 4 for extent 8x4x1"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_truncated_dfd_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 48, 160);
    write_u32(&mut bytes, 52, 32);

    let error = import_container_error("truncated-dfd.ktx2", bytes);

    assert!(
        error.contains("ktx2 data format descriptor requires at least 192 bytes, got 176"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_invalid_data_format_descriptor_total_size() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 2] = [
        (
            "short-dfd.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 48, KTX2_HEADER_SIZE as u32 + 96);
                write_u32(bytes, 52, 2);
                bytes.resize(KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE * 4 + 2, 0);
            },
            "ktx2 data format descriptor length must be at least 4 bytes when present",
        ),
        (
            "dfd-total-size-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 48, KTX2_HEADER_SIZE as u32 + 96);
                write_u32(bytes, 52, 8);
                bytes.resize(KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE * 4 + 8, 0);
                write_u32(bytes, KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE * 4, 4);
            },
            "ktx2 data format descriptor total size 4 must equal dfdByteLength 8",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_truncated_key_value_data_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 56, 168);
    write_u32(&mut bytes, 60, 16);

    let error = import_container_error("truncated-kvd.ktx2", bytes);

    assert!(
        error.contains("ktx2 key/value data requires at least 184 bytes, got 176"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_truncated_supercompression_global_data_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u64(&mut bytes, 64, 176);
    write_u64(&mut bytes, 72, 8);

    let error = import_container_error("truncated-sgd.ktx2", bytes);

    assert!(
        error.contains("ktx2 supercompression global data requires at least 184 bytes, got 176"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_truncated_level_payload_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u64(&mut bytes, KTX2_HEADER_SIZE, 172);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 8);

    let error = import_container_error("truncated-level.ktx2", bytes);

    assert!(
        error.contains("ktx2 level 0 payload requires at least 180 bytes, got 176"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_invalid_level_uncompressed_byte_lengths() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 4] = [
        (
            "none-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_NONE);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 4);
                bytes.resize(184, 0);
            },
            "ktx2 level 0 uncompressed byte length must equal byte length when supercompression is none",
        ),
        (
            "basislz-uncompressed.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_BASIS_LZ);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 8);
                bytes.resize(184, 0);
            },
            "ktx2 level 0 uncompressed byte length must be 0 for BasisLZ supercompression",
        ),
        (
            "zstd-zero-uncompressed.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 0);
                bytes.resize(184, 0);
            },
            "ktx2 level 0 uncompressed byte length must be nonzero for supercompression scheme 2 when byte length is nonzero",
        ),
        (
            "image-count-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 10);
                bytes.resize(184, 0);
            },
            "ktx2 level 0 uncompressed byte length must be divisible by image count 12, got 10",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_level_payload_inside_level_index() {
    let mut bytes = tiny_ktx2_bytes();
    write_u64(&mut bytes, KTX2_HEADER_SIZE, KTX2_HEADER_SIZE as u64);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 4);

    let error = import_container_error("level-inside-index.ktx2", bytes);

    assert!(
        error.contains("ktx2 level 0 payload starts inside header or level index"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_overlapping_level_payload_ranges() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
    write_u64(&mut bytes, KTX2_HEADER_SIZE, 176);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 16);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 24);
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE,
        184,
    );
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE + 8,
        16,
    );
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE + 16,
        24,
    );
    bytes.resize(200, 0);

    let error = import_container_error("overlapping-levels.ktx2", bytes);

    assert!(
        error.contains("ktx2 level 1 payload overlaps level 0 payload"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_metadata_ranges_inside_level_index() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 3] = [
        (
            "dfd-inside-index.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 48, KTX2_HEADER_SIZE as u32);
                write_u32(bytes, 52, 4);
            },
            "ktx2 data format descriptor starts inside header or level index",
        ),
        (
            "kvd-inside-index.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 56, KTX2_HEADER_SIZE as u32);
                write_u32(bytes, 60, 4);
            },
            "ktx2 key/value data starts inside header or level index",
        ),
        (
            "sgd-inside-index.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u64(bytes, 64, KTX2_HEADER_SIZE as u64);
                write_u64(bytes, 72, 4);
            },
            "ktx2 supercompression global data starts inside header or level index",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_overlapping_metadata_ranges() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 48, 176);
    write_u32(&mut bytes, 52, 8);
    write_u32(&mut bytes, 56, 180);
    write_u32(&mut bytes, 60, 8);
    bytes.resize(188, 0);
    write_u32(&mut bytes, 176, 8);

    let error = import_container_error("overlapping-metadata.ktx2", bytes);

    assert!(
        error.contains("ktx2 key/value data overlaps ktx2 data format descriptor"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_metadata_range_overlapping_level_payload() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
    write_u64(&mut bytes, KTX2_HEADER_SIZE, 176);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 16);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 24);
    write_u32(&mut bytes, 48, 184);
    write_u32(&mut bytes, 52, 8);
    bytes.resize(192, 0);
    write_u32(&mut bytes, 184, 8);

    let error = import_container_error("metadata-overlaps-level.ktx2", bytes);

    assert!(
        error.contains("ktx2 data format descriptor overlaps ktx2 level 0 payload"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_reads_3d_dimension() {
    let imported = import_container_fixture("volume.ktx2", tiny_ktx2_3d_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.height, 8);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.dimension, RenderImageDimension::D3);
            assert_eq!(descriptor.depth_or_array_layers, 5);
            assert_eq!(descriptor.array_layer_count, 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx2_3d_container_keeps_depth_separate_from_array_layers() {
    let mut bytes = ktx2_layer_face_bytes(2, 6);
    write_u32(&mut bytes, 28, 5);
    let imported = import_container_fixture("volume-array.ktx2", bytes);

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
fn ktx2_container_importer_rejects_3d_texture_without_height() {
    let mut bytes = tiny_ktx2_3d_bytes();
    write_u32(&mut bytes, 24, 0);

    let error = import_container_error("volume-without-height.ktx2", bytes);

    assert!(
        error.contains("ktx2 3d texture height must be nonzero when depth is nonzero"),
        "unexpected error: {error}"
    );
}
