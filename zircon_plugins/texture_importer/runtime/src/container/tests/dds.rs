use super::common::*;
use crate::container::support::{
    DDSCAPS2_CUBEMAP, DDSCAPS2_CUBEMAP_NEGATIVEX, DDSCAPS2_CUBEMAP_POSITIVEX, DDSCAPS_COMPLEX,
    DDSCAPS_MIPMAP, DDSCAPS_TEXTURE, DDSD_LINEARSIZE, DDSD_REQUIRED_FLAGS, DDS_DIMENSION_TEXTURE2D,
    DDS_RESOURCE_MISC_TEXTURECUBE,
};
use zircon_runtime::asset::{ImportedAsset, TexturePayload};
use zircon_runtime::core::framework::render::RenderImageDimension;

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
fn dds_dx10_container_importer_rejects_truncated_bc2_payload_using_dxgi_block_size() {
    let mut bytes = tiny_dds_bytes();
    bytes.resize(148 + 32, 0);
    bytes[84..88].copy_from_slice(b"DX10");
    write_u32(&mut bytes, 128, 74);
    write_u32(&mut bytes, 132, DDS_DIMENSION_TEXTURE2D);
    write_u32(&mut bytes, 140, 1);

    let error = import_container_error("truncated-dx10-bc2.dds", bytes);

    assert!(
        error.contains("dds compressed mip chain payload requires at least 212 bytes, got 180"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_dx10_container_importer_accepts_typeless_payload_using_dxgi_block_size() {
    for (dxgi_format, expected_bytes_per_block) in [
        (70, 8_usize),
        (73, 16),
        (76, 16),
        (79, 8),
        (82, 16),
        (94, 16),
        (97, 16),
    ] {
        let mut bytes = tiny_dds_bytes();
        write_u32(&mut bytes, 16, 4);
        write_u32(&mut bytes, 28, 1);
        bytes.resize(148 + expected_bytes_per_block, 0);
        bytes[84..88].copy_from_slice(b"DX10");
        write_u32(&mut bytes, 128, dxgi_format);
        write_u32(&mut bytes, 132, DDS_DIMENSION_TEXTURE2D);
        write_u32(&mut bytes, 140, 1);

        let imported = import_container_fixture("dx10-typeless.dds", bytes);

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(
                    texture.render_image_descriptor().format,
                    format!("dds/dxgi-{dxgi_format}")
                );
                match texture.payload {
                    TexturePayload::Container { bytes, .. } => {
                        assert_eq!(bytes.len(), 148 + expected_bytes_per_block);
                    }
                    other => panic!("unexpected texture payload: {other:?}"),
                }
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }
}

#[test]
fn dds_dx10_container_importer_rejects_short_header() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    bytes.truncate(147);

    let error = import_container_error("short-dx10-header.dds", bytes);

    assert!(
        error.contains("dds dx10 header requires at least 148 bytes, got 147"),
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
fn dds_dx10_container_importer_rejects_unknown_dxgi_format() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    write_u32(&mut bytes, 128, 0);

    let error = import_container_error("unknown-dxgi-format.dds", bytes);

    assert!(
        error.contains("dds dx10 format must not be DXGI_FORMAT_UNKNOWN"),
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
fn dds_dx10_container_importer_rejects_dual_cubemap_sources() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    write_u32(&mut bytes, 136, DDS_RESOURCE_MISC_TEXTURECUBE);

    let error = import_container_error("dual-cubemap-sources.dds", bytes);

    assert!(
        error.contains(
            "dds dx10 cubemap must be declared by legacy caps2 or DX10 texturecube flag, not both"
        ),
        "unexpected error: {error}"
    );
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
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
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
fn dds_container_importer_rejects_mipmap_count_without_mipmap_caps() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 108, DDSCAPS_TEXTURE);

    let error = import_container_error("missing-mipmap-caps.dds", bytes);

    assert!(
        error.contains(
            "dds caps must include DDSCAPS_MIPMAP and DDSCAPS_COMPLEX when multiple mip levels are declared"
        ),
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
fn dds_container_importer_rejects_cubemap_face_flags_without_cubemap_bit() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    write_u32(
        &mut bytes,
        112,
        DDSCAPS2_CUBEMAP_POSITIVEX | DDSCAPS2_CUBEMAP_NEGATIVEX,
    );

    let error = import_container_error("face-flags-without-cubemap.dds", bytes);

    assert!(
        error.contains("dds cubemap face caps2 flags require DDSCAPS2_CUBEMAP"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_cubemap_without_complex_caps() {
    let mut bytes = tiny_dds_dx10_cubemap_array_bytes();
    write_u32(&mut bytes, 108, DDSCAPS_TEXTURE | DDSCAPS_MIPMAP);

    let error = import_container_error("cubemap-without-complex-caps.dds", bytes);

    assert!(
        error.contains("dds caps must include DDSCAPS_COMPLEX when cubemap faces are declared"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_dx10_container_importer_rejects_misc_texturecube_without_complex_caps() {
    let mut bytes = dds_dx10_cubemap_array_bytes(2);
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
    write_u32(&mut bytes, 28, 1);
    write_u32(&mut bytes, 108, DDSCAPS_TEXTURE | DDSCAPS_MIPMAP);
    write_u32(&mut bytes, 112, 0);
    write_u32(&mut bytes, 136, DDS_RESOURCE_MISC_TEXTURECUBE);

    let error = import_container_error("misc-cube-without-complex-caps.dds", bytes);

    assert!(
        error.contains(
            "dds caps must include DDSCAPS_COMPLEX when cubemap faces are declared or DX10 texturecube flag is set"
        ),
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
