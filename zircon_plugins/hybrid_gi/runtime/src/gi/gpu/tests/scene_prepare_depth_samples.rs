use crate::hybrid_gi::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCacheDepthSourceSample,
    HybridGiPrepareSurfaceCachePageContent,
};

use super::*;

#[test]
fn surface_cache_depth_samples_prefer_scene_depth_source_samples_over_bounds_fallback() {
    let snapshot = single_atlas_slot_snapshot(0);
    let mut inputs = HybridGiPrepareExecutionInputs::default();
    inputs.scene_surface_cache_depth_source_samples =
        vec![HybridGiPrepareSurfaceCacheDepthSourceSample {
            page_id: 7,
            atlas_slot_id: 0,
            depth_rgba: [17, 19, 23, 255],
        }];
    inputs.scene_surface_cache_page_contents = vec![HybridGiPrepareSurfaceCachePageContent {
        page_id: 7,
        owner_card_id: 7,
        atlas_slot_id: 0,
        capture_slot_id: 0,
        bounds_center: Vec3::new(16.0, 0.0, 0.0),
        bounds_radius: 0.5,
        atlas_sample_rgba: [64, 96, 128, 255],
        capture_sample_rgba: [0, 0, 0, 0],
    }];

    assert_eq!(
        surface_cache_depth_samples(&snapshot, &inputs),
        vec![(0, [17, 19, 23, 255])]
    );
}

#[test]
fn surface_cache_depth_samples_keep_bounds_fallback_when_scene_depth_source_is_absent() {
    let snapshot = single_atlas_slot_snapshot(1);
    let mut inputs = HybridGiPrepareExecutionInputs::default();
    inputs.scene_card_capture_requests = vec![HybridGiPrepareCardCaptureRequest {
        card_id: 7,
        page_id: 7,
        atlas_slot_id: 1,
        capture_slot_id: 0,
        bounds_center: Vec3::new(0.0, 0.0, 3.0),
        bounds_radius: 1.0,
    }];

    assert_eq!(
        surface_cache_depth_samples(&snapshot, &inputs),
        vec![(1, depth_rgba_from_bounds(Vec3::new(0.0, 0.0, 3.0), 1.0))]
    );
}

fn single_atlas_slot_snapshot(slot_id: u32) -> HybridGiScenePrepareResourcesSnapshot {
    let slot_count = slot_id + 1;
    HybridGiScenePrepareResourcesSnapshot::new(
        0,
        Vec::new(),
        vec![slot_id],
        Vec::new(),
        slot_count,
        0,
        (
            SURFACE_CACHE_DEPTH_TILE_EXTENT * SURFACE_CACHE_DEPTH_ATLAS_COLUMNS,
            SURFACE_CACHE_DEPTH_TILE_EXTENT
                * slot_count.div_ceil(SURFACE_CACHE_DEPTH_ATLAS_COLUMNS),
        ),
        (0, 0),
        0,
    )
}
