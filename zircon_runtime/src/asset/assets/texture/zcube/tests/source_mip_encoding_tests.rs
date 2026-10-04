use crate::asset::AssetUri;
use crate::core::framework::render::{
    encode_rgba16f_texels, source_cubemap_sample_count, SourceCubemapMipChain,
};

use super::{
    encode_source_cubemap_zcube_rgba16f_mips, encode_source_cubemap_zcube_rgba16f_mips_owned,
    texture_asset_from_source_cubemap_zcube, texture_asset_from_source_cubemap_zcube_mips,
    texture_asset_from_source_cubemap_zcube_rgba16f_mips, ZcubeSourceCubemapError,
};

fn uri() -> AssetUri {
    AssetUri::parse("res://generated/probes/atrium.zcube").unwrap()
}

#[test]
fn raw_source_mips_encode_identically_without_constructing_filtered_outputs() {
    let face_size = 2;
    let mip_count = 2;
    let source_texels = (0..source_cubemap_sample_count(face_size, mip_count))
        .map(|index| [index as f32, 0.5, 0.25, 1.0])
        .collect::<Vec<_>>();
    let filtered = SourceCubemapMipChain::new(
        face_size,
        mip_count,
        source_texels.clone(),
        1,
        1,
        vec![[0.0, 0.0, 0.0, 1.0]; 6],
    );

    let legacy = texture_asset_from_source_cubemap_zcube(uri(), &filtered);
    let raw =
        texture_asset_from_source_cubemap_zcube_mips(uri(), face_size, mip_count, &source_texels)
            .unwrap();

    assert_eq!(raw, legacy);
}

#[test]
fn canonical_rgba16f_source_bytes_encode_identically_without_f32_reexpansion() {
    let face_size = 2;
    let mip_count = 2;
    let source_texels = (0..source_cubemap_sample_count(face_size, mip_count))
        .map(|index| [index as f32, 0.5, 0.25, 1.0])
        .collect::<Vec<_>>();
    let expected =
        texture_asset_from_source_cubemap_zcube_mips(uri(), face_size, mip_count, &source_texels)
            .unwrap();

    let actual = texture_asset_from_source_cubemap_zcube_rgba16f_mips(
        uri(),
        face_size,
        mip_count,
        &encode_rgba16f_texels(&source_texels),
    )
    .unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn canonical_rgba16f_source_bytes_reject_incomplete_payloads() {
    let face_size = 2;
    let mip_count = 2;
    let expected = source_cubemap_sample_count(face_size, mip_count) * 8;
    let error = texture_asset_from_source_cubemap_zcube_rgba16f_mips(
        uri(),
        face_size,
        mip_count,
        &vec![0; expected - 1],
    )
    .unwrap_err();

    assert_eq!(
        error,
        ZcubeSourceCubemapError::InvalidPayloadLength {
            expected,
            actual: expected - 1,
        }
    );
}

#[test]
fn raw_source_mips_reject_incomplete_layout_before_encoding() {
    let error = texture_asset_from_source_cubemap_zcube_mips(
        uri(),
        2,
        2,
        &vec![[0.0; 4]; source_cubemap_sample_count(2, 2) - 1],
    )
    .unwrap_err();

    assert_eq!(
        error,
        ZcubeSourceCubemapError::SourceTexelCountMismatch {
            expected: source_cubemap_sample_count(2, 2),
            actual: source_cubemap_sample_count(2, 2) - 1,
        }
    );
}

#[test]
fn raw_source_mips_require_the_complete_source_pyramid() {
    let error = texture_asset_from_source_cubemap_zcube_mips(
        uri(),
        4,
        2,
        &vec![[0.0; 4]; source_cubemap_sample_count(4, 2)],
    )
    .unwrap_err();

    assert_eq!(
        error,
        ZcubeSourceCubemapError::InvalidMipCount {
            face_size: 4,
            expected: 3,
            actual: 2,
        }
    );
}

#[test]
fn owned_canonical_encoder_matches_borrowed_encoder_without_requantization() {
    let face_size = 2;
    let mip_count = 2;
    let source = vec![0x5a; source_cubemap_sample_count(face_size, mip_count) * 8];
    let borrowed = encode_source_cubemap_zcube_rgba16f_mips(face_size, mip_count, &source).unwrap();
    let owned =
        encode_source_cubemap_zcube_rgba16f_mips_owned(face_size, mip_count, source).unwrap();
    assert_eq!(owned, borrowed);
}
