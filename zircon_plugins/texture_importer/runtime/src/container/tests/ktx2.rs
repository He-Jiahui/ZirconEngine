use super::common::*;
use std::io::Write;

use crate::container::support::{
    KTX2_HEADER_SIZE, KTX2_LEVEL_INDEX_ENTRY_SIZE, KTX2_SUPERCOMPRESSION_BASIS_LZ,
    KTX2_SUPERCOMPRESSION_NONE, KTX2_SUPERCOMPRESSION_ZLIB, KTX2_SUPERCOMPRESSION_ZSTANDARD,
};
use zircon_runtime::asset::{ImportedAsset, TexturePayload, TextureUploadSupport};
use zircon_runtime::core::framework::render::RenderImageDimension;

#[test]
fn ktx2_container_importer_reads_layers_faces_and_mips() {
    let imported = import_container_fixture("array.ktx2", tiny_ktx2_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.height, 16);
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
                    assert_eq!(format, "ktx2/vk-37/supercompression-0");
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
            "ktx2/vk-0/supercompression-1",
        ),
        (
            KTX2_SUPERCOMPRESSION_ZSTANDARD,
            "ktx2/vk-37/supercompression-0",
        ),
        (KTX2_SUPERCOMPRESSION_ZLIB, "ktx2/vk-37/supercompression-0"),
    ];

    for (scheme, expected_format) in cases {
        let mut bytes = tiny_ktx2_bytes();
        write_u32(&mut bytes, 44, scheme);
        if scheme == KTX2_SUPERCOMPRESSION_BASIS_LZ {
            write_u32(&mut bytes, 12, 0);
        }

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
fn ktx2_container_importer_decodes_standard_supercompression_for_gpu_upload() {
    let decoded_level = vec![0x6d; 128];
    let cases = [
        (
            KTX2_SUPERCOMPRESSION_ZSTANDARD,
            zstd::stream::encode_all(decoded_level.as_slice(), 0)
                .expect("zstd fixture encoding should succeed"),
        ),
        (KTX2_SUPERCOMPRESSION_ZLIB, zlib_encode(&decoded_level)),
    ];

    for (scheme, compressed_level) in cases {
        let source = ktx2_standard_supercompressed_bc1_bytes(
            scheme,
            compressed_level.as_slice(),
            decoded_level.len(),
        );
        let source_len = source.len();
        let imported = import_container_fixture("standard-supercompression.ktx2", source);

        let ImportedAsset::Texture(texture) = imported else {
            panic!("KTX2 importer should produce a texture asset");
        };
        let TexturePayload::Container { format, bytes, .. } = &texture.payload else {
            panic!("KTX2 importer should preserve a container texture payload");
        };
        assert_eq!(format, "ktx2/vk-133/supercompression-0");
        assert_eq!(&bytes[44..48], &0_u32.to_le_bytes());
        assert!(
            texture
                .upload_readiness(TextureUploadSupport::all_compressed())
                .is_ready(),
            "decoded KTX2 payload should be ready for BC upload"
        );
        let level_offset = usize::try_from(read_u64(&bytes, KTX2_HEADER_SIZE))
            .expect("decoded level offset should fit the host address space");
        let level_length = usize::try_from(read_u64(&bytes, KTX2_HEADER_SIZE + 8))
            .expect("decoded level length should fit the host address space");
        assert_eq!(level_length, decoded_level.len());
        assert_eq!(
            &bytes[level_offset..level_offset + level_length],
            decoded_level.as_slice()
        );
        assert!(
            bytes.len() < source_len + decoded_level.len(),
            "rewritten KTX2 container should not retain compressed and decoded payloads"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_invalid_standard_supercompressed_payload() {
    let decoded_level = vec![0x6d; 128];
    let cases = [
        (KTX2_SUPERCOMPRESSION_ZSTANDARD, vec![0x00, 0x01, 0x02]),
        (KTX2_SUPERCOMPRESSION_ZLIB, vec![0x00, 0x01, 0x02]),
    ];

    for (scheme, compressed_level) in cases {
        let source = ktx2_standard_supercompressed_bc1_bytes(
            scheme,
            compressed_level.as_slice(),
            decoded_level.len(),
        );
        let error = import_container_error("invalid-standard-supercompression.ktx2", source);

        assert!(
            error.contains("decode KTX2"),
            "invalid standard supercompressed payload should fail closed, got: {error}"
        );
    }
}

fn zlib_encode(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(bytes)
        .expect("zlib fixture encoding should accept the level bytes");
    encoder
        .finish()
        .expect("zlib fixture encoding should finish successfully")
}

fn ktx2_standard_supercompressed_bc1_bytes(
    scheme: u32,
    compressed_level: &[u8],
    decoded_level_len: usize,
) -> Vec<u8> {
    let mut bytes = ktx2_layer_face_level_bytes(1, 1, 1);
    write_u32(&mut bytes, 12, 133);
    write_u32(&mut bytes, 44, scheme);
    let payload_offset = bytes
        .len()
        .checked_add(7)
        .map(|value| value & !7)
        .expect("fixture payload alignment should fit usize");
    bytes.resize(payload_offset, 0);
    bytes.extend_from_slice(compressed_level);
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE,
        u64::try_from(payload_offset).expect("fixture offset should fit u64"),
    );
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE + 8,
        u64::try_from(compressed_level.len()).expect("fixture compressed length should fit u64"),
    );
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE + 16,
        u64::try_from(decoded_level_len).expect("fixture decoded length should fit u64"),
    );
    bytes
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("fixture range is valid"),
    )
}

#[test]
fn ktx2_container_importer_rejects_basislz_with_concrete_vk_format() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_BASIS_LZ);

    let error = import_container_error("basislz-concrete-format.ktx2", bytes);

    assert!(
        error.contains("ktx2 BasisLZ supercompression requires vkFormat 0, got 37"),
        "unexpected error: {error}"
    );
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
    write_u32(&mut bytes, 36, 1);
    write_u32(&mut bytes, 40, 5);

    let error = import_container_error("overlarge-mip-count.ktx2", bytes);

    assert!(
        error.contains("ktx2 mip level count 5 exceeds maximum 4 for extent 8x4x1"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_missing_data_format_descriptor() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 48, 0);
    write_u32(&mut bytes, 52, 0);

    let error = import_container_error("missing-dfd.ktx2", bytes);

    assert!(
        error.contains("ktx2 data format descriptor must be present"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_truncated_dfd_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 48, KTX2_AFTER_DEFAULT_DFD_OFFSET as u32);
    write_u32(&mut bytes, 52, KTX2_DEFAULT_DFD_BYTE_LENGTH as u32);

    let error = import_container_error("truncated-dfd.ktx2", bytes);

    assert!(
        error.contains("ktx2 data format descriptor requires at least 232 bytes, got 204"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_invalid_data_format_descriptor_structure() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 12] = [
        (
            "short-dfd.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 52, 12);
            },
            "ktx2 data format descriptor length must be at least 16 bytes",
        ),
        (
            "dfd-total-size-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET, 24);
            },
            "ktx2 data format descriptor total size 24 must equal dfdByteLength 28",
        ),
        (
            "dfd-offset-misaligned.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 48, (KTX2_DEFAULT_DFD_OFFSET + 2) as u32);
            },
            "ktx2 data format descriptor offset must be 4-byte aligned, got 178",
        ),
        (
            "dfd-length-misaligned.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 52, (KTX2_DEFAULT_DFD_BYTE_LENGTH + 2) as u32);
            },
            "ktx2 data format descriptor length must be 4-byte aligned, got 30",
        ),
        (
            "dfd-block-size-too-small.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 8, (20 << 16) | 2);
            },
            "ktx2 data format descriptor basic descriptor block size 20 must be at least 24 bytes and 16-byte sample aligned",
        ),
        (
            "dfd-block-version-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 8, (24 << 16) | 1);
            },
            "ktx2 data format descriptor block 0 version must be 2, got 1",
        ),
        (
            "dfd-transfer-out-of-range.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 12, 20 << 16);
            },
            "ktx2 data format descriptor transfer function 20 is not supported",
        ),
        (
            "dfd-block-size-misaligned.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 8, (26 << 16) | 2);
            },
            "ktx2 data format descriptor basic descriptor block size 26 must be at least 24 bytes and 16-byte sample aligned",
        ),
        (
            "dfd-block-size-exceeds-length.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 8, (40 << 16) | 2);
            },
            "ktx2 data format descriptor basic descriptor block size 40 exceeds dfdByteLength 28",
        ),
        (
            "dfd-vendor-type-nonzero.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 4, 1);
            },
            "ktx2 data format descriptor block 0 vendor/type word must be 0",
        ),
        (
            "dfd-block-chain-trailing-bytes.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 52, 32);
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET, 32);
                bytes.resize(KTX2_DEFAULT_DFD_OFFSET + 32, 0);
            },
            "ktx2 data format descriptor block chain leaves 4 trailing descriptor bytes",
        ),
        (
            "dfd-second-block-exceeds-remaining.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 52, 52);
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET, 52);
                bytes.resize(KTX2_DEFAULT_DFD_OFFSET + 52, 0);
                write_u32(bytes, KTX2_DEFAULT_DFD_OFFSET + 28 + 4, (40 << 16) | 2);
            },
            "ktx2 data format descriptor block 1 size 40 exceeds remaining DFD descriptor bytes 24",
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
fn ktx2_container_importer_rejects_truncated_supercompression_global_data_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u64(&mut bytes, 64, KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64);
    write_u64(&mut bytes, 72, 8);

    let error = import_container_error("truncated-sgd.ktx2", bytes);

    assert!(
        error.contains("ktx2 supercompression global data requires at least 216 bytes, got 204"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_supercompression_global_data_when_none() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_NONE);
    write_u64(&mut bytes, 64, KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64);
    write_u64(&mut bytes, 72, 8);
    bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);

    let error = import_container_error("sgd-without-supercompression.ktx2", bytes);

    assert!(
        error.contains(
            "ktx2 supercompression global data must be empty when supercompression is none"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_basislz_payload_without_supercompression_global_data() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 12, 0);
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_BASIS_LZ);
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE,
        KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64,
    );
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 8);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 0);
    bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);

    let error = import_container_error("basislz-payload-without-sgd.ktx2", bytes);

    assert!(
        error.contains("ktx2 BasisLZ level payloads require supercompression global data"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_supercompression_global_data_for_zstd_or_zlib() {
    let cases = [
        (
            KTX2_SUPERCOMPRESSION_ZSTANDARD,
            "ktx2 supercompression global data must be empty for supercompression scheme 2",
        ),
        (
            KTX2_SUPERCOMPRESSION_ZLIB,
            "ktx2 supercompression global data must be empty for supercompression scheme 3",
        ),
    ];

    for (scheme, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        write_u32(&mut bytes, 44, scheme);
        write_u64(&mut bytes, 64, KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64);
        write_u64(&mut bytes, 72, 8);
        bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);

        let error = import_container_error("sgd-for-inflate-supercompression.ktx2", bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_truncated_level_payload_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE,
        KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64,
    );
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 12);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 12);

    let error = import_container_error("truncated-level.ktx2", bytes);

    assert!(
        error.contains("ktx2 level 0 payload requires at least 220 bytes, got 204"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_misaligned_level_payload_offset() {
    let mut bytes = tiny_ktx2_bytes();
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE,
        (KTX2_AFTER_DEFAULT_DFD_OFFSET + 1) as u64,
    );
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 12);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 12);
    bytes.resize(KTX2_AFTER_DEFAULT_DFD_OFFSET + 9, 0);

    let error = import_container_error("misaligned-level.ktx2", bytes);

    assert!(
        error.contains("ktx2 level 0 payload offset must be 8-byte aligned, got 205"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_invalid_level_uncompressed_byte_lengths() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 7] = [
        (
            "empty-level-offset.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u64(
                    bytes,
                    KTX2_HEADER_SIZE,
                    KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64,
                );
            },
            "ktx2 level 0 payload offset must be 0 when byte length is 0",
        ),
        (
            "empty-level-uncompressed.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 8);
            },
            "ktx2 level 0 uncompressed byte length must be 0 when byte length is 0",
        ),
        (
            "none-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_NONE);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 4);
                bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);
            },
            "ktx2 level 0 uncompressed byte length must equal byte length when supercompression is none",
        ),
        (
            "basislz-uncompressed.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 12, 0);
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_BASIS_LZ);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 8);
                bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);
            },
            "ktx2 level 0 uncompressed byte length must be 0 for BasisLZ supercompression",
        ),
        (
            "zstd-zero-uncompressed.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 0);
                bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);
            },
            "ktx2 level 0 uncompressed byte length must be nonzero for supercompression scheme 2 when byte length is nonzero",
        ),
        (
            "image-count-mismatch.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 8);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 10);
                bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8, 0);
            },
            "ktx2 level 0 uncompressed byte length must be divisible by image count 12, got 10",
        ),
        (
            "supercompressed-uncompressed-smaller.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
                write_u64(
                    bytes,
                    KTX2_HEADER_SIZE,
                    KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64,
                );
                write_u64(bytes, KTX2_HEADER_SIZE + 8, 16);
                write_u64(bytes, KTX2_HEADER_SIZE + 16, 12);
                bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 16, 0);
            },
            "ktx2 level 0 uncompressed byte length must be at least byte length for supercompression scheme 2",
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
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 12);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 12);

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
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE,
        KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64,
    );
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 16);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 24);
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE + KTX2_LEVEL_INDEX_ENTRY_SIZE,
        (KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 8) as u64,
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
    bytes.resize(KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + 24, 0);

    let error = import_container_error("overlapping-levels.ktx2", bytes);

    assert!(
        error.contains("ktx2 level 1 payload overlaps level 0 payload"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_reads_3d_dimension() {
    let imported = import_container_fixture("volume.ktx2", tiny_ktx2_3d_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.height, 16);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.dimension, RenderImageDimension::D3);
            assert_eq!(descriptor.depth_or_array_layers, 5);
            assert_eq!(descriptor.array_layer_count, 1);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx2_container_importer_rejects_1d_cubemap_header() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 24, 0);

    let error = import_container_error("strip-cubemap.ktx2", bytes);

    assert!(
        error.contains("ktx2 cubemap faces must be 2d and square, got 16x0"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_non_square_cubemap_header() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 24, 8);

    let error = import_container_error("non-square-cubemap.ktx2", bytes);

    assert!(
        error.contains("ktx2 cubemap faces must be 2d and square, got 16x8"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_3d_array_header() {
    let mut bytes = ktx2_layer_face_bytes(2, 1);
    write_u32(&mut bytes, 28, 5);

    let error = import_container_error("volume-array.ktx2", bytes);

    assert!(
        error.contains("ktx2 3d textures must not declare array layers"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_3d_cubemap_header() {
    let mut bytes = tiny_ktx2_3d_bytes();
    write_u32(&mut bytes, 36, 6);

    let error = import_container_error("volume-cubemap.ktx2", bytes);

    assert!(
        error.contains("ktx2 cubemap textures must not declare 3d depth"),
        "unexpected error: {error}"
    );
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
