use super::common::*;
use crate::asset::{AssetUri, TextureAsset, TextureUploadSupport};

#[test]
fn texture_upload_readiness_rejects_dds_header_mip_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let missing_mip_flag = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-default-mips.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_bytes("DXT1", 8),
        2,
        1,
    );
    assert_eq!(
        missing_mip_flag
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let stale_mip_count = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-header-mips.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_mip_bytes("DXT1", 2, 16),
        1,
        1,
    );
    assert_eq!(
        stale_mip_count
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_rejects_dds_header_layer_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let stale_legacy_cubemap = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-legacy-cubemap.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_cubemap_bytes("DXT1", 48),
        1,
        1,
    );
    assert_eq!(
        stale_legacy_cubemap
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let stale_dx10_array = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-dx10-array.dds").unwrap(),
        4,
        4,
        "dds/dxgi-98",
        dds_dx10_array_bytes(98, 2, false, 32),
        1,
        1,
    );
    assert_eq!(
        stale_dx10_array
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/dxgi-98 is not upload-ready")
    );

    let valid_dx10_array = TextureAsset::new_container(
        AssetUri::parse("res://textures/dx10-array.dds").unwrap(),
        4,
        4,
        "dds/dxgi-98",
        dds_dx10_array_bytes(98, 2, false, 32),
        1,
        2,
    );
    assert_eq!(
        valid_dx10_array
            .upload_readiness(support)
            .unsupported_reason(),
        Some("compressed texture array/cubemap upload is not implemented")
    );
}

#[test]
fn texture_upload_readiness_rejects_dds_main_header_structural_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let invalid_headers = [
        ("header-size", 4, 120),
        (
            "required-flags",
            8,
            DDSD_CAPS | DDSD_WIDTH | DDSD_PIXELFORMAT | DDSD_LINEARSIZE,
        ),
        ("missing-linear-size", 8, DDSD_REQUIRED_FLAGS),
        (
            "pitch-and-linear-size",
            8,
            DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE | DDSD_PITCH,
        ),
        ("zero-linear-size", 20, 0),
        ("raw-depth", 24, 1),
        ("pixel-format-size", 76, 28),
        ("pixel-format-fourcc-flag", 80, 0),
        ("missing-texture-caps", 108, 0),
        ("volume-caps2", 112, DDSCAPS2_VOLUME),
    ];
    for (name, offset, value) in invalid_headers {
        let mut bytes = dds_legacy_bytes("DXT1", 8);
        write_u32_le(&mut bytes, offset, value);
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/{name}.dds")).unwrap(),
            4,
            4,
            "dds/DXT1",
            bytes,
            1,
            1,
        );
        assert_eq!(
            texture.upload_readiness(support).unsupported_reason(),
            Some("texture container format dds/DXT1 is not upload-ready")
        );
    }

    let mut missing_mip_caps = dds_legacy_mip_bytes("DXT1", 2, 16);
    write_u32_le(&mut missing_mip_caps, 108, DDSCAPS_TEXTURE);
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/missing-mip-caps.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        missing_mip_caps,
        2,
        1,
    );
    assert_eq!(
        texture.upload_readiness(support).unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let zero_mip_count = TextureAsset::new_container(
        AssetUri::parse("res://textures/zero-mip-count.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_mip_bytes("DXT1", 0, 16),
        1,
        1,
    );
    assert_eq!(
        zero_mip_count
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let mut missing_cubemap_face = dds_legacy_cubemap_bytes("DXT1", 48);
    write_u32_le(&mut missing_cubemap_face, 112, DDSCAPS2_CUBEMAP);
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/missing-cubemap-face.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        missing_cubemap_face,
        1,
        6,
    );
    assert_eq!(
        texture.upload_readiness(support).unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let mut face_without_cubemap = dds_legacy_bytes("DXT1", 8);
    write_u32_le(&mut face_without_cubemap, 112, DDSCAPS2_CUBEMAP_POSITIVEX);
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/face-without-cubemap.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        face_without_cubemap,
        1,
        1,
    );
    assert_eq!(
        texture.upload_readiness(support).unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_rejects_dds_dx10_structural_header_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let invalid_headers = [
        ("resource-dimension", 132, 4),
        ("misc-flag-bits", 136, 0x8),
        ("array-size-zero", 140, 0),
        ("misc-flags2-reserved", 144, 0x8),
        ("alpha-mode", 144, 5),
    ];
    for (name, offset, value) in invalid_headers {
        let mut bytes = dds_dx10_bytes(98, 16);
        write_u32_le(&mut bytes, offset, value);
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/{name}.dds")).unwrap(),
            4,
            4,
            "dds/dxgi-98",
            bytes,
            1,
            1,
        );
        assert_eq!(
            texture.upload_readiness(support).unsupported_reason(),
            Some("texture container format dds/dxgi-98 is not upload-ready")
        );
    }

    let mut duplicate_cubemap_declaration = dds_dx10_array_bytes(98, 1, true, 16);
    write_u32_le(&mut duplicate_cubemap_declaration, 112, 0x0000_fe00);
    write_u32_le(&mut duplicate_cubemap_declaration, 108, 0x0000_0008);
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/duplicate-cubemap-dds-dx10.dds").unwrap(),
        4,
        4,
        "dds/dxgi-98",
        duplicate_cubemap_declaration,
        1,
        6,
    );
    assert_eq!(
        texture.upload_readiness(support).unsupported_reason(),
        Some("texture container format dds/dxgi-98 is not upload-ready")
    );

    let missing_complex_caps = TextureAsset::new_container(
        AssetUri::parse("res://textures/dx10-cubemap-missing-complex-caps.dds").unwrap(),
        4,
        4,
        "dds/dxgi-98",
        dds_dx10_array_bytes(98, 1, true, 16),
        1,
        6,
    );
    assert_eq!(
        missing_complex_caps
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/dxgi-98 is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_rejects_dds_descriptor_header_format_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut bad_magic = dds_legacy_bytes("DXT1", 8);
    bad_magic[0] = 0;
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-magic.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        bad_magic,
        1,
        1,
    );
    assert_eq!(
        texture.upload_readiness(support).unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let mismatched_fourcc = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-fourcc.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_bytes("DXT5", 16),
        1,
        1,
    );
    assert_eq!(
        mismatched_fourcc
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let dxgi_without_dx10_fourcc = TextureAsset::new_container(
        AssetUri::parse("res://textures/dxgi-without-dx10-fourcc.dds").unwrap(),
        4,
        4,
        "dds/dxgi-98",
        dds_legacy_bytes("DXT5", 16),
        1,
        1,
    );
    assert_eq!(
        dxgi_without_dx10_fourcc
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/dxgi-98 is not upload-ready")
    );

    let mismatched_dxgi = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-dxgi.dds").unwrap(),
        4,
        4,
        "dds/dxgi-98",
        dds_dx10_bytes(95, 16),
        1,
        1,
    );
    assert_eq!(
        mismatched_dxgi
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format dds/dxgi-98 is not upload-ready")
    );
}
