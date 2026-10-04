use crate::core::framework::render::{
    RenderImageColorSpace, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
};

use super::*;

fn descriptor(usage: TextureUsageHint) -> TextureAssetDescriptor {
    let mut descriptor = TextureAssetDescriptor::decoded_rgba8_for_import_usage(usage);
    descriptor.metadata.mip_policy = TextureMipPolicy::GenerateOffline;
    descriptor
}

#[test]
fn offline_build_packs_complete_chain_and_preserves_base_payload() {
    let mut rgba = Vec::with_capacity(84);
    rgba.extend((0_u8..64).map(|value| value.saturating_mul(3)));
    let base = rgba.clone();
    let pointer = rgba.as_ptr();

    let texture = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/packed.png").unwrap(),
        4,
        4,
        rgba,
        descriptor(TextureUsageHint::Data),
    )
    .unwrap();

    assert_eq!(texture.texture_descriptor().mip_count, 3);
    assert_eq!(texture.rgba.len(), 84);
    assert_eq!(&texture.rgba[..64], base.as_slice());
    assert_eq!(texture.rgba.as_ptr(), pointer);
    assert!(texture
        .upload_readiness(super::super::TextureUploadSupport::uncompressed_only())
        .is_ready());
}

#[test]
fn srgb_box_filter_averages_in_linear_space() {
    let mut descriptor = descriptor(TextureUsageHint::Albedo);
    descriptor.metadata.mip_filter = TextureMipFilter::Box;
    descriptor.color_space = RenderImageColorSpace::Srgb;
    descriptor.metadata.color_space = RenderImageColorSpace::Srgb;
    let texture = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/srgb.png").unwrap(),
        2,
        2,
        vec![
            0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
        ],
        descriptor,
    )
    .unwrap();

    assert_eq!(&texture.rgba[16..20], &[188, 188, 188, 255]);
}

#[test]
fn normal_mip_is_renormalized_after_dx_to_gl_projection() {
    let mut descriptor = descriptor(TextureUsageHint::Normal);
    descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceDx;
    let texture = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/normal.png").unwrap(),
        2,
        2,
        vec![
            255, 128, 128, 255, 128, 255, 128, 255, 255, 128, 128, 255, 128, 255, 128, 255,
        ],
        descriptor,
    )
    .unwrap();

    let mip = &texture.rgba[16..20];
    assert!(mip[0] >= 217 && mip[1] <= 38);
    assert!((126..=130).contains(&mip[2]));
    assert_eq!(mip[3], 255);
    assert_eq!(
        texture.texture_descriptor().metadata.normal_convention,
        TextureNormalConvention::TangentSpaceGl
    );
}

#[test]
fn decoded_build_rejects_truncated_base_payload() {
    let error = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/truncated.png").unwrap(),
        2,
        2,
        vec![0; 15],
        descriptor(TextureUsageHint::Data),
    )
    .unwrap_err();

    assert!(error.to_string().contains("expected 16 base bytes"));
}

#[test]
fn decoded_build_rejects_mismatched_array_shape() {
    let mut descriptor = descriptor(TextureUsageHint::Data);
    descriptor.depth_or_array_layers = 2;
    let error = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/array.png").unwrap(),
        1,
        1,
        vec![0; 8],
        descriptor,
    )
    .unwrap_err();

    assert!(error.to_string().contains("single-layer 2d"));
}

#[test]
fn decoded_build_rejects_matching_array_layers_without_an_array_mip_owner() {
    let mut descriptor = descriptor(TextureUsageHint::Data);
    descriptor.depth_or_array_layers = 2;

    let error = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/array.png").unwrap(),
        1,
        1,
        vec![0; 8],
        descriptor,
    )
    .unwrap_err();

    assert!(error.to_string().contains("single-layer 2d"));
}

#[test]
fn decoded_build_rejects_cube_without_a_seam_aware_mip_owner() {
    let mut descriptor = descriptor(TextureUsageHint::Data);
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = 6;

    let error = build_decoded_rgba8_texture(
        AssetUri::parse("res://textures/cube.png").unwrap(),
        1,
        1,
        vec![0; 24],
        descriptor,
    )
    .unwrap_err();

    assert!(error.to_string().contains("single-layer 2d"));
}
