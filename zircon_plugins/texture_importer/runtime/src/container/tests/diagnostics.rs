use super::common::*;
use crate::container::support::{KTX2_HEADER_SIZE, KTX2_IDENTIFIER};

#[test]
fn container_importer_reports_invalid_header_diagnostics() {
    let cases = [
        ("broken.dds", vec![0; 128], "dds header missing DDS magic"),
        (
            "broken.ktx",
            vec![0; 64],
            "ktx header missing KTX 1 identifier",
        ),
        (
            "broken.ktx2",
            vec![0; KTX2_HEADER_SIZE],
            "ktx2 header missing KTX 2 identifier",
        ),
        ("broken.astc", vec![0; 16], "astc header missing ASTC magic"),
    ];

    for (path, bytes, expected) in cases {
        let error = import_container_error(path, bytes);
        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }

    let mut short_ktx2_level_index = vec![0; KTX2_HEADER_SIZE];
    short_ktx2_level_index[0..12].copy_from_slice(KTX2_IDENTIFIER);
    write_u32(&mut short_ktx2_level_index, 20, 16);
    write_u32(&mut short_ktx2_level_index, 24, 8);
    write_u32(&mut short_ktx2_level_index, 40, 4);
    let error = import_container_error("short-index.ktx2", short_ktx2_level_index);
    assert!(
        error.contains("ktx2 level index requires at least 176 bytes, got 80"),
        "expected short level index diagnostic in `{error}`"
    );
}

#[test]
fn astc_container_importer_rejects_zero_block_or_extent_fields() {
    let cases = [
        (4, "astc block x must be nonzero"),
        (5, "astc block y must be nonzero"),
        (6, "astc block z must be nonzero"),
        (7, "astc width must be nonzero"),
        (10, "astc height must be nonzero"),
        (13, "astc depth must be nonzero"),
    ];

    for (offset, expected) in cases {
        let mut bytes = tiny_astc_bytes();
        bytes[offset] = 0;
        if matches!(offset, 7 | 10 | 13) {
            bytes[offset + 1] = 0;
            bytes[offset + 2] = 0;
        }
        let error = import_container_error("zero.astc", bytes);
        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn container_importer_reports_layer_count_overflow_diagnostics() {
    let cases = [
        (
            "overflow.dds",
            dds_dx10_cubemap_array_bytes(u32::MAX),
            "dds dx10 array layer count overflows u32",
        ),
        (
            "overflow.ktx",
            {
                let mut bytes = ktx1_layer_face_bytes(u32::MAX, 6);
                write_u32(&mut bytes, 40, 32);
                bytes
            },
            "ktx array layer count overflows u32",
        ),
        (
            "overflow.ktx2",
            ktx2_layer_face_bytes(u32::MAX, 6),
            "ktx2 array layer count overflows u32",
        ),
    ];

    for (path, bytes, expected) in cases {
        let error = import_container_error(path, bytes);
        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn container_importer_rejects_invalid_ktx_face_counts() {
    let cases = [
        (
            "zero-face.ktx",
            ktx1_layer_face_bytes(1, 0),
            "ktx face count must be 1 for ordinary textures or 6 for cubemaps, got 0",
        ),
        (
            "invalid-face.ktx",
            ktx1_layer_face_bytes(1, 2),
            "ktx face count must be 1 for ordinary textures or 6 for cubemaps, got 2",
        ),
        (
            "zero-face.ktx2",
            ktx2_layer_face_bytes(1, 0),
            "ktx2 face count must be 1 for ordinary textures or 6 for cubemaps, got 0",
        ),
        (
            "invalid-face.ktx2",
            ktx2_layer_face_bytes(1, 5),
            "ktx2 face count must be 1 for ordinary textures or 6 for cubemaps, got 5",
        ),
    ];

    for (path, bytes, expected) in cases {
        let error = import_container_error(path, bytes);
        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}
