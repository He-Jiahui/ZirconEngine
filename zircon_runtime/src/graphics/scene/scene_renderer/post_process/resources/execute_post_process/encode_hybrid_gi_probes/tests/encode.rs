use super::*;
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiPreparedProbeRtLighting,
    RenderPreparedRuntimeSidebands,
};
use crate::core::math::UVec2;
use crate::graphics::ViewportRenderFrame;
use crate::scene::world::World;

#[test]
fn hybrid_gi_probe_encoder_returns_no_resources_when_disabled() {
    let frame = ViewportRenderFrame::from_extract(
        World::new().to_render_frame_extract(),
        UVec2::new(160, 120),
    );

    let (_, probe_count) = encode_hybrid_gi_probes(&frame, UVec2::new(160, 120), false);

    assert_eq!(probe_count, 0);
}

#[test]
fn hybrid_gi_probe_encoder_requires_prepared_scene_probe_sideband() {
    let frame = ViewportRenderFrame::from_extract(
        hybrid_gi_scene_representation_extract(),
        UVec2::new(160, 120),
    );

    let (_, probe_count) = encode_hybrid_gi_probes(&frame, UVec2::new(160, 120), true);

    assert_eq!(probe_count, 0);
}

#[test]
fn hybrid_gi_sideband_lookup_preserves_reordered_duplicate_first_match() {
    let prepared = RenderHybridGiPreparedFrame {
        probe_scene_data: vec![scene_data(9, 90), scene_data(7, 70), scene_data(7, 71)],
        probe_rt_lighting_rgb: vec![
            rt_lighting(9, [90, 0, 0]),
            rt_lighting(7, [70, 0, 0]),
            rt_lighting(7, [71, 0, 0]),
        ],
        ..RenderHybridGiPreparedFrame::default()
    };

    let lookup = PreparedProbeSidebandLookup::new(&prepared);

    assert!(!lookup.scene_data_is_canonical);
    assert!(!lookup.rt_lighting_is_canonical);
    assert_eq!(
        lookup.scene_data(7).map(|entry| entry.position_x_q),
        Some(70)
    );
    assert_eq!(
        lookup.rt_lighting(7).map(|entry| entry.rt_lighting_rgb),
        Some([70, 0, 0])
    );
}

#[test]
fn optimization_batch_20260830dv_hybrid_gi_sidebands_use_canonical_binary_lookup() {
    let prepared = RenderHybridGiPreparedFrame {
        probe_scene_data: vec![scene_data(3, 30), scene_data(7, 70), scene_data(9, 90)],
        probe_rt_lighting_rgb: vec![
            rt_lighting(3, [30, 0, 0]),
            rt_lighting(7, [70, 0, 0]),
            rt_lighting(9, [90, 0, 0]),
        ],
        ..RenderHybridGiPreparedFrame::default()
    };
    let lookup = PreparedProbeSidebandLookup::new(&prepared);

    assert!(lookup.scene_data_is_canonical);
    assert!(lookup.rt_lighting_is_canonical);
    assert_eq!(
        lookup.scene_data(7).map(|entry| entry.position_x_q),
        Some(70)
    );
    assert_eq!(
        lookup.rt_lighting(7).map(|entry| entry.rt_lighting_rgb),
        Some([70, 0, 0])
    );

    let source = include_str!("../encode.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(production.contains("binary_search_by_key"));
    assert!(production.contains("strictly_increasing_by_key"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dv_hybrid_gi_sideband_lookup_evidence() {
    const FRAME_COUNT: usize = 32_768;
    const PROBE_COUNT: usize = MAX_HYBRID_GI_PROBES;
    const MARKER: &str = "RUNTIME531_HYBRID_GI_SIDEBAND_BINARY_LOOKUP_BENCH_V1";

    let legacy_checks_per_frame = PROBE_COUNT
        .saturating_mul(PROBE_COUNT.saturating_add(1))
        .saturating_mul(2)
        / 2;
    let comparisons_per_lookup = usize::BITS as usize - PROBE_COUNT.leading_zeros() as usize;
    let indexed_checks_per_frame = PROBE_COUNT
        .saturating_sub(1)
        .saturating_mul(2)
        .saturating_add(
            PROBE_COUNT
                .saturating_mul(comparisons_per_lookup)
                .saturating_mul(2),
        );
    let legacy_candidate_checks = FRAME_COUNT.saturating_mul(legacy_checks_per_frame);
    let indexed_candidate_checks = FRAME_COUNT.saturating_mul(indexed_checks_per_frame);
    let reduction_bps = legacy_candidate_checks
        .saturating_sub(indexed_candidate_checks)
        .saturating_mul(10_000)
        / legacy_candidate_checks.max(1);

    assert!(
        indexed_candidate_checks.saturating_mul(100) <= legacy_candidate_checks.saturating_mul(70)
    );
    println!(
        "{MARKER} frames={FRAME_COUNT} probes={PROBE_COUNT} \
             legacy_candidate_checks={legacy_candidate_checks} \
             indexed_candidate_checks_upper_bound={indexed_candidate_checks} \
             comparisons_per_lookup={comparisons_per_lookup} reduction_bps={reduction_bps}"
    );
}

#[test]
fn hybrid_gi_probe_encoder_projects_prepared_runtime_screen_probe_sideband() {
    let frame = ViewportRenderFrame::from_extract(
        hybrid_gi_scene_representation_extract(),
        UVec2::new(160, 120),
    )
    .with_prepared_runtime_sidebands(
        RenderPreparedRuntimeSidebands::default().with_hybrid_gi_prepared_frame(Some(
            RenderHybridGiPreparedFrame {
                resident_probes: vec![RenderHybridGiPreparedProbe {
                    probe_id: 7,
                    slot: 0,
                    stable_instance_key: 77,
                    source_mask: crate::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
                    dynamic_weight_q8: u8::MAX,
                    ray_budget: 1,
                    irradiance_rgb: [32, 40, 48],
                }],
                probe_scene_data: vec![RenderHybridGiPreparedProbeSceneData {
                    probe_id: 7,
                    position_x_q: 2048,
                    position_y_q: 2048,
                    position_z_q: 2048,
                    radius_q: 96,
                }],
                probe_rt_lighting_rgb: vec![RenderHybridGiPreparedProbeRtLighting {
                    probe_id: 7,
                    rt_lighting_rgb: [240, 64, 32],
                }],
                ..RenderHybridGiPreparedFrame::default()
            },
        )),
    );

    let (probes, probe_count) = encode_hybrid_gi_probes(&frame, UVec2::new(160, 120), true);

    assert_eq!(probe_count, 1);
    assert!(probes[0].screen_uv_and_radius[2] > 0.0);
    assert_eq!(probes[0].irradiance_and_intensity[0], 32.0_f32 / 255.0);
    assert_eq!(
        probes[0].hierarchy_rt_lighting_rgb_and_weight[0],
        240.0_f32 / 255.0
    );
    assert!(probes[0].hierarchy_rt_lighting_rgb_and_weight[3] > 0.0);
}

fn hybrid_gi_scene_representation_extract() -> RenderFrameExtract {
    let world = World::new();
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(160, 120));
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        trace_budget: 2,
        card_budget: 1,
        voxel_budget: 1,
        ..RenderHybridGiExtract::default()
    });
    extract
}

fn scene_data(probe_id: u32, position_x_q: u32) -> RenderHybridGiPreparedProbeSceneData {
    RenderHybridGiPreparedProbeSceneData {
        probe_id,
        position_x_q,
        position_y_q: 2048,
        position_z_q: 2048,
        radius_q: 96,
    }
}

fn rt_lighting(probe_id: u32, rt_lighting_rgb: [u8; 3]) -> RenderHybridGiPreparedProbeRtLighting {
    RenderHybridGiPreparedProbeRtLighting {
        probe_id,
        rt_lighting_rgb,
    }
}
