use crate::asset::{
    AssetUri, TextureAsset, TextureAssetDescriptor, decode_ibl_pmrem_rgba16f_texture,
    texture_asset_from_ibl_bake_artifact_pmrem, texture_asset_from_source_cubemap_zcube,
};
use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, ProceduralSkyParams, RenderImageDimension,
    build_source_cubemap_from_equirect, decode_rgba16f_texels,
};
use crate::core::resource::ResourceId;

use super::super::upload::{ReflectionProbeAssetError, validate_probe_pmrem_texture};

#[test]
fn render_probe_source_cubemap_mips_are_rejected_as_pmrem() {
    let source = build_source_cubemap_from_equirect(128, |u, v| [u, v, 0.5, 1.0]);
    let texture = texture_asset_from_source_cubemap_zcube(
        AssetUri::parse("mem://probe/source.zcube").expect("asset uri"),
        &source,
    );
    let id = ResourceId::from_stable_label("probe:source-zcube");

    let error = validate_probe_pmrem_texture(id, &texture).expect_err("source mips are not PMREM");

    assert!(matches!(
        error,
        ReflectionProbeAssetError::SourceCubemapRequiresPrefiltering { cubemap } if cubemap == id
    ));
}

#[test]
fn render_probe_accepts_current_rgba16f_pmrem_and_preserves_hdr_range() {
    let source =
        build_source_cubemap_from_equirect(128, |u, v| [2.0 + u * 4.0, 0.5 + v, 0.25, 1.0]);
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM);
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &source, None)
        .expect("valid PMREM payload");
    let texture = texture_asset_from_ibl_bake_artifact_pmrem(
        AssetUri::parse("mem://probe/current-pmrem.zpmrem").expect("asset uri"),
        &IblBakeArtifactBlob::from_payload(payload),
    )
    .expect("current PMREM texture");
    let id = ResourceId::from_stable_label("probe:current-rgba16f-pmrem");

    validate_probe_pmrem_texture(id, &texture).expect("current RGBA16F PMREM is accepted");
    let bytes = decode_ibl_pmrem_rgba16f_texture(&texture).expect("decode PMREM texture");
    let texels = decode_rgba16f_texels(bytes);

    assert!(
        texels.iter().any(|texel| texel[0] > 1.0),
        "probe PMREM must preserve HDR values above the normalized rgba8 range"
    );
}

#[test]
fn render_probe_rejects_rgba8_cubemap_as_non_hdr_pmrem() {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = 6;
    descriptor.array_layer_count = 6;
    descriptor.mip_count = 8;
    let byte_len = (0..8)
        .map(|mip| {
            let size = (128_u32 >> mip).max(1) as usize;
            size * size * 4 * 6
        })
        .sum();
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("mem://probe/rgba8-cubemap.png").expect("asset uri"),
        128,
        128,
        vec![255; byte_len],
    )
    .with_descriptor(descriptor);
    let id = ResourceId::from_stable_label("probe:rgba8-cubemap");

    let error = validate_probe_pmrem_texture(id, &texture)
        .expect_err("rgba8 cubemap must not be accepted as HDR PMREM");

    assert!(matches!(error, ReflectionProbeAssetError::Payload { cubemap } if cubemap == id));
}
