use crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest;
use zircon_runtime::core::math::Vec3;

use super::*;

#[test]
fn surface_cache_trace_tile_ray_count_scales_with_hybrid_gi_quality_budget() {
    let snapshot = HybridGiScenePrepareResourcesSnapshot::new(
        1,
        Vec::new(),
        vec![3],
        Vec::new(),
        4,
        0,
        (32, 8),
        (0, 0),
        0,
    );
    let mut inputs = HybridGiPrepareExecutionInputs::default();
    inputs.scene_card_capture_requests = vec![HybridGiPrepareCardCaptureRequest {
        card_id: 11,
        page_id: 22,
        atlas_slot_id: 3,
        capture_slot_id: 0,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
    }];

    for (quality_tracing_budget, expected_ray_count) in
        [(Some(8), 4), (Some(16), 8), (Some(32), 16), (None, 8)]
    {
        let tiles = probe_trace_tiles(&snapshot, &inputs, quality_tracing_budget);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].3, expected_ray_count);
    }
}

#[test]
fn probe_trace_tile_generation_pipeline_is_device_owned_not_frame_created() {
    let frame_source = include_str!("../scene_prepare_trace_tiles.rs");
    let construct_source = include_str!("../../../../new/construct/construct.rs");

    let layout_factory = ["create_probe_trace_tile_generation_", "bind_group_layout"].concat();
    assert!(construct_source.contains(&layout_factory));
    let pipeline_factory = ["create_probe_trace_tile_generation_", "pipeline"].concat();
    let frame_pipeline_use = [
        "pass.set_pipeline(&resources.",
        "probe_trace_tile_generation_pipeline);",
    ]
    .concat();
    let frame_layout_factory = ["fn ", &layout_factory, "("].concat();
    let frame_pipeline_factory = ["fn ", &pipeline_factory, "("].concat();

    assert!(construct_source.contains(
        "let probe_trace_tile_generation_bind_group_layout =\n            create_probe_trace_tile_generation_bind_group_layout(device);"
    ));
    assert!(construct_source.contains(
        "let probe_trace_tile_generation_pipeline = create_probe_trace_tile_generation_pipeline("
    ));
    assert!(construct_source.contains("probe_trace_tile_generation_bind_group_layout,"));
    assert!(construct_source.contains("probe_trace_tile_generation_pipeline,"));
    assert!(frame_source.contains(&frame_pipeline_use));
    assert!(!frame_source.contains(&frame_layout_factory));
    assert!(!frame_source.contains(&frame_pipeline_factory));
}
