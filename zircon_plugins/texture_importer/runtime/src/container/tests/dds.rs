use super::super::*;
use super::common::*;

#[test]
fn dds_container_importer_preserves_compressed_payload() {
    let imported = import_container_fixture("albedo.dds", tiny_dds_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 8);
            assert_eq!(texture.height, 4);
            assert!(texture.rgba.is_empty());
            assert_eq!(texture.render_image_descriptor().format, "dds/DXT1");
            match texture.payload {
                TexturePayload::Container {
                    format,
                    bytes,
                    mip_count,
                    array_layers,
                } => {
                    assert_eq!(format, "dds/DXT1");
                    assert_eq!(bytes.len(), 160);
                    assert_eq!(mip_count, 3);
                    assert_eq!(array_layers, 1);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn dds_dx10_container_importer_reads_cubemap_array_layers() {
    let imported = import_container_fixture("skybox.dds", tiny_dds_dx10_cubemap_array_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 32);
            assert_eq!(texture.height, 16);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "dds/dxgi-98");
            assert_eq!(descriptor.dimension, RenderImageDimension::D2);
            assert_eq!(descriptor.depth_or_array_layers, 12);
            assert_eq!(descriptor.mip_count, 5);
            assert_eq!(descriptor.array_layer_count, 12);
            match texture.payload {
                TexturePayload::Container {
                    format,
                    bytes,
                    mip_count,
                    array_layers,
                } => {
                    assert_eq!(format, "dds/dxgi-98");
                    assert_eq!(bytes.len(), 8596);
                    assert_eq!(mip_count, 5);
                    assert_eq!(array_layers, 12);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn dds_container_importer_rejects_truncated_compressed_mip_chain_payload() {
    let mut bytes = tiny_dds_bytes();
    bytes.truncate(144);

    let error = import_container_error("truncated-dxt1.dds", bytes);

    assert!(
        error.contains("dds compressed mip chain payload requires at least 160 bytes, got 144"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_dx10_container_importer_rejects_truncated_compressed_mip_chain_payload() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    bytes.truncate(6292);

    let error = import_container_error("truncated-dx10.dds", bytes);

    assert!(
        error.contains("dds compressed mip chain payload requires at least 8596 bytes, got 6292"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_dx10_container_importer_rejects_zero_array_size() {
    let error = import_container_error("empty-array.dds", dds_dx10_cubemap_array_bytes(0));

    assert!(
        error.contains("dds dx10 array size must be nonzero"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_dx10_container_importer_rejects_unsupported_resource_dimension() {
    let cases = [
        (0, "dds dx10 resource dimension must be texture2d, got 0"),
        (4, "dds dx10 resource dimension must be texture2d, got 4"),
    ];

    for (resource_dimension, expected) in cases {
        let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
        write_u32(&mut bytes, 132, resource_dimension);

        let error = import_container_error("unsupported-dimension.dds", bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn dds_dx10_container_importer_reads_misc_texturecube_flag() {
    let mut bytes = dds_dx10_cubemap_array_bytes(2);
    write_u32(&mut bytes, 112, 0);
    write_u32(&mut bytes, 136, DDS_RESOURCE_MISC_TEXTURECUBE);

    let imported = import_container_fixture("dx10-misc-cube.dds", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.depth_or_array_layers, 12);
            assert_eq!(descriptor.array_layer_count, 12);
            match texture.payload {
                TexturePayload::Container { array_layers, .. } => {
                    assert_eq!(array_layers, 12);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn dds_dx10_container_importer_rejects_unsupported_misc_flag_bits() {
    let cases = [
        (0x0000_0008, "0x00000008"),
        (DDS_RESOURCE_MISC_TEXTURECUBE | 0x0000_0010, "0x00000010"),
    ];

    for (misc_flag, expected_bits) in cases {
        let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
        write_u32(&mut bytes, 136, misc_flag);

        let error = import_container_error("unsupported-misc-flag.dds", bytes);

        assert!(
            error.contains(&format!(
                "dds dx10 misc flag contains unsupported bits {expected_bits}"
            )),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn dds_dx10_container_importer_rejects_invalid_misc_flags2() {
    let cases = [
        (
            0x0000_0008,
            "dds dx10 miscFlags2 contains reserved bits 0x00000008",
        ),
        (0x0000_0005, "dds dx10 alpha mode must be 0..=4, got 5"),
    ];

    for (misc_flags2, expected) in cases {
        let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
        write_u32(&mut bytes, 144, misc_flags2);

        let error = import_container_error("invalid-misc-flags2.dds", bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn dds_container_importer_ignores_mip_count_without_mipmap_flag() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS);
    write_u32(&mut bytes, 28, 7);

    let imported = import_container_fixture("single-level.dds", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.mip_count, 1);
            match texture.payload {
                TexturePayload::Container { mip_count, .. } => {
                    assert_eq!(mip_count, 1);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn dds_container_importer_rejects_declared_zero_mip_count() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 28, 0);

    let error = import_container_error("zero-mips.dds", bytes);

    assert!(
        error.contains("dds mip map count must be nonzero when DDSD_MIPMAPCOUNT is set"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_mip_count_larger_than_extent_chain() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 28, 5);

    let error = import_container_error("too-many-mips.dds", bytes);

    assert!(
        error.contains("dds mip map count 5 exceeds maximum 4 for extent 8x4"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_incomplete_cubemap_face_flags() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    write_u32(
        &mut bytes,
        112,
        DDSCAPS2_CUBEMAP | DDSCAPS2_CUBEMAP_POSITIVEX,
    );

    let error = import_container_error("partial-cubemap.dds", bytes);

    assert!(
        error.contains("dds cubemap caps2 must include all six face flags"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_texture_caps() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 108, 0);

    let error = import_container_error("missing-texture-caps.dds", bytes);

    assert!(
        error.contains("dds caps must include DDSCAPS_TEXTURE"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_required_header_flags() {
    let cases = [
        (
            DDSD_REQUIRED_FLAGS & !DDSD_CAPS,
            "dds header flags must include DDSD_CAPS",
        ),
        (
            DDSD_REQUIRED_FLAGS & !DDSD_HEIGHT,
            "dds header flags must include DDSD_HEIGHT",
        ),
        (
            DDSD_REQUIRED_FLAGS & !DDSD_WIDTH,
            "dds header flags must include DDSD_WIDTH",
        ),
        (
            DDSD_REQUIRED_FLAGS & !DDSD_PIXELFORMAT,
            "dds header flags must include DDSD_PIXELFORMAT",
        ),
    ];

    for (flags, expected) in cases {
        let mut bytes = tiny_dds_bytes();
        write_u32(&mut bytes, 8, flags);

        let error = import_container_error("missing-header-flag.dds", bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn dds_container_importer_rejects_fourcc_without_pixel_format_flag() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 80, 0);

    let error = import_container_error("fourcc-without-flag.dds", bytes);

    assert!(
        error.contains("dds FourCC field is nonzero but DDPF_FOURCC flag is missing"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_fourcc_flag_without_fourcc_field() {
    let mut bytes = tiny_dds_bytes();
    bytes[84..88].copy_from_slice(&[0, 0, 0, 0]);

    let error = import_container_error("flag-without-fourcc.dds", bytes);

    assert!(
        error.contains("dds pixel format flags include DDPF_FOURCC but FourCC field is empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_volume_headers() {
    let cases: [(&str, fn(&mut Vec<u8>)); 2] = [
        ("volume-depth.dds", |bytes: &mut Vec<u8>| {
            write_u32(bytes, 24, 4);
        }),
        ("volume-caps.dds", |bytes: &mut Vec<u8>| {
            write_u32(bytes, 112, DDSCAPS2_VOLUME);
        }),
    ];

    for (path, mutate) in cases {
        let mut bytes = tiny_dds_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains("dds volume textures are not supported by container importer yet"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn dds_container_importer_rejects_malformed_fourcc_tokens() {
    let cases: [(&str, [u8; 4], &str); 2] = [
        (
            "non-ascii-fourcc.dds",
            [b'D', b'X', 0xff, b'1'],
            "dds fourcc must contain printable ASCII bytes",
        ),
        (
            "embedded-nul-fourcc.dds",
            [b'D', b'X', 0, b'1'],
            "dds fourcc must not contain embedded NUL bytes",
        ),
    ];

    for (path, fourcc, expected) in cases {
        let mut bytes = tiny_dds_bytes();
        bytes[84..88].copy_from_slice(&fourcc);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}
