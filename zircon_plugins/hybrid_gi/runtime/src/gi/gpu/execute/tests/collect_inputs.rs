use std::collections::BTreeMap;

use crate::hybrid_gi::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareProbe, HybridGiPrepareRadianceCacheConsume,
    HybridGiPrepareRadianceCacheUpdate, HybridGiPrepareSurfaceCachePageContent,
    HybridGiResolveProbeSceneData, HybridGiResolveTraceRegionSceneData,
};

use super::*;

#[test]
fn collect_inputs_preserves_scene_prepare_and_runtime_sideband_contracts() {
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 7,
            slot: 0,
            stable_instance_key: 0,
            source_mask: zircon_runtime::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
            dynamic_weight_q8: u8::MAX,
            ray_budget: 32,
            irradiance_rgb: [8, 16, 24],
        }],
        scheduled_trace_region_ids: vec![9],
        ..HybridGiPrepareFrame::default()
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_scene_data(BTreeMap::from([(
            7,
            HybridGiResolveProbeSceneData::new(2112, 2048, 2048, 96),
        )]))
        .with_trace_region_scene_data(BTreeMap::from([(
            9,
            HybridGiResolveTraceRegionSceneData::new(2112, 2048, 2048, 192, 128, [64, 96, 128]),
        )]))
        .build();
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
            card_id: 11,
            page_id: 22,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(1.0, 2.0, 3.0),
            bounds_radius: 0.5,
        }],
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 22,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(1.0, 2.0, 3.0),
            bounds_radius: 0.5,
            atlas_sample_rgba: [10, 20, 30, 255],
            capture_sample_rgba: [40, 50, 60, 255],
        }],
        radiance_cache_updates: vec![HybridGiPrepareRadianceCacheUpdate {
            slot: 3,
            generation: 9,
            radiance_rgb: [40, 50, 60],
            confidence_q8: 200,
            reuse_committed_radiance: false,
        }],
        radiance_cache_consumes: vec![HybridGiPrepareRadianceCacheConsume {
            probe_id: 7,
            generation: 9,
            slots: [3; 8],
            weights_q16: [u16::MAX, 0, 0, 0, 0, 0, 0, 0],
        }],
        ..HybridGiScenePrepareFrame::default()
    };

    let inputs = collect_inputs(
        &prepare,
        Some(&runtime),
        Some(&scene_prepare),
        &scene_prepare.radiance_cache_updates,
        &scene_prepare.radiance_cache_consumes,
        Arc::<[(u64, RenderMeshBounds)]>::from([]),
        Arc::<[RenderMeshSnapshot]>::from([]),
        &[],
        &[],
        &[],
    );

    assert_eq!(inputs.resident_probe_inputs.len(), 1);
    assert_eq!(inputs.resident_probe_inputs[0].position_x_q, 2112);
    assert_eq!(inputs.trace_region_inputs.len(), 1);
    assert_eq!(inputs.trace_region_inputs[0].region_id, 9);
    assert_eq!(inputs.scene_card_capture_descriptor_count, 1);
    assert_eq!(inputs.scene_surface_cache_page_contents.len(), 1);
    assert_eq!(inputs.scene_surface_cache_depth_source_samples.len(), 1);
    assert_eq!(inputs.radiance_cache_update_inputs.len(), 1);
    assert_eq!(inputs.radiance_cache_update_inputs[0].slot, 3);
    assert_eq!(inputs.radiance_cache_update_inputs[0].generation_low, 9);
    assert_eq!(
        inputs.radiance_cache_update_inputs[0]
            .radiance_confidence
            .to_le_bytes(),
        [40, 50, 60, 200]
    );
    assert_eq!(inputs.radiance_cache_consume_inputs.len(), 1);
    assert_eq!(inputs.radiance_cache_consume_inputs[0].probe_id, 7);
    assert_eq!(
        inputs.radiance_cache_consume_inputs[0].resident_probe_index,
        0
    );
    assert_eq!(inputs.radiance_cache_consume_inputs[0].slots, [3; 8]);
    assert_eq!(
        inputs.radiance_cache_consume_inputs[0].weights_q16[0],
        u32::from(u16::MAX)
    );
}
