use super::*;
use crate::core::framework::render::{source_cubemap_sample_count, ProceduralSkyParams};

#[test]
fn header_round_trips_descriptor() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let descriptor =
        IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);

    let encoded = IblBakeArtifactHeader::from_descriptor(descriptor).encode();

    assert_eq!(
        IblBakeArtifactHeader::decode(&encoded)
            .unwrap()
            .descriptor(),
        descriptor
    );
}

#[test]
fn header_v4_wire_matches_the_golden_fixture() {
    let bake_key = IblBakeKey {
        source_kind: 0x1122_3344,
        source_revision: 0x0102_0304_0506_0708,
        horizon_color: [0x1112_1314, 0x2122_2324, 0x3132_3334, 0x4142_4344],
        zenith_color: [0x5152_5354, 0x6162_6364, 0x7172_7374, 0x8182_8384],
        ground_color: [0x9192_9394, 0xa1a2_a3a4, 0xb1b2_b3b4, 0xc1c2_c3c4],
        source_hash: [0xd1d2_d3d4, 0xe1e2_e3e4, 0xf1f2_f3f4, 0x0102_0304],
    };
    let request = IblBakeArtifactRequest::new(bake_key, 64, 6)
        .with_pmrem_layout(128, 8)
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let descriptor = IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request)
        .with_algorithm_version(0x8877_6655_4433_2211);
    let expected = [
        0x5a, 0x52, 0x49, 0x42, 0x4c, 0x42, 0x41, 0x4b, 0x04, 0x00, 0x00, 0x00, 0x11, 0x22, 0x33,
        0x44, 0x55, 0x66, 0x77, 0x88, 0x40, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x80, 0x00,
        0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x44,
        0x33, 0x22, 0x11, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x14, 0x13, 0x12, 0x11,
        0x24, 0x23, 0x22, 0x21, 0x34, 0x33, 0x32, 0x31, 0x44, 0x43, 0x42, 0x41, 0x54, 0x53, 0x52,
        0x51, 0x64, 0x63, 0x62, 0x61, 0x74, 0x73, 0x72, 0x71, 0x84, 0x83, 0x82, 0x81, 0x94, 0x93,
        0x92, 0x91, 0xa4, 0xa3, 0xa2, 0xa1, 0xb4, 0xb3, 0xb2, 0xb1, 0xc4, 0xc3, 0xc2, 0xc1, 0xd4,
        0xd3, 0xd2, 0xd1, 0xe4, 0xe3, 0xe2, 0xe1, 0xf4, 0xf3, 0xf2, 0xf1, 0x04, 0x03, 0x02, 0x01,
    ];

    assert_eq!(expected.len(), IBL_BAKE_ARTIFACT_HEADER_SIZE);
    assert_eq!(
        IblBakeArtifactHeader::from_descriptor(descriptor).encode(),
        expected
    );
    assert_eq!(
        IblBakeArtifactHeader::decode(&expected)
            .expect("V4 golden fixture must decode")
            .descriptor(),
        descriptor
    );
}

#[test]
fn descriptor_recipe_identity_keeps_cpu_and_runtime_integrators_distinct() {
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        128,
        8,
    );
    let asset = IblBakeArtifactDescriptor::current_for_request(&request);
    let runtime = IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request);
    let stale = runtime.with_algorithm_version(IBL_BAKE_ALGORITHM_VERSION - 1);

    assert_eq!(
        asset.recipe_identity(),
        CANONICAL_IBL_BAKE_RECIPE.asset_recipe_identity()
    );
    assert_eq!(
        runtime.recipe_identity(),
        CANONICAL_IBL_BAKE_RECIPE.runtime_recipe_identity()
    );
    assert_ne!(asset.recipe_identity(), runtime.recipe_identity());
    assert_ne!(stale.recipe_identity(), runtime.recipe_identity());
    assert!(!stale.is_current_runtime_cache_for(&request));
}

#[test]
fn sh9_only_payload_does_not_require_pmrem_layout_match() {
    let cubemap = SourceCubemapMipChain::new(
        4,
        3,
        vec![[0.25, 0.5, 0.75, 1.0]; source_cubemap_sample_count(4, 3)],
        4,
        3,
        vec![[0.25, 0.5, 0.75, 1.0]; source_cubemap_sample_count(4, 3)],
    );
    let request =
        IblBakeArtifactRequest::new(ProceduralSkyParams::default_gradient().ibl_bake_key(), 4, 3)
            .with_required_contents(IblBakeArtifactContents::SH9);
    let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);

    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &cubemap, None)
        .expect("SH9 serialization depends on coefficients, not PMREM texture layout");

    assert_eq!(payload.bytes().len(), IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES);
}
