use super::*;
use zircon_runtime::core::framework::render::{
    RenderHybridGiProbeTraceDiagnosticRecord, RenderHybridGiTraceCostCounters,
    RenderHybridGiTraceFallbackReason, RenderHybridGiTraceIntersectionSource,
    RenderHybridGiTraceLightingSource,
};

#[test]
fn neutral_outputs_project_hybrid_gi_gpu_readback() {
    let mut scene_prepare = HybridGiScenePrepareResourcesSnapshot::new(
        2,
        vec![9],
        vec![1, 2],
        vec![3],
        8,
        4,
        (64, 32),
        (16, 16),
        6,
    );
    scene_prepare.store_texture_slot_rgba_samples(
        vec![(1, [10, 20, 30, 255])],
        vec![(3, [40, 50, 60, 255])],
    );
    scene_prepare.store_voxel_resource_samples(
        vec![(9, [70, 80, 90, 255])],
        vec![(9, 0b1011)],
        vec![(9, 2, [100, 110, 120, 255])],
        vec![(9, 2, 4)],
        vec![(9, 2, 77)],
        vec![(9, 2, [130, 140, 150, 255])],
    );
    scene_prepare.store_surface_cache_depth_samples(vec![(1, [96, 96, 96, 255])]);
    scene_prepare.store_probe_trace_tiles(vec![(0, 9, 2, 32)], [1, 1, 1]);
    scene_prepare.store_probe_trace_diagnostics(vec![RenderHybridGiProbeTraceDiagnosticRecord {
        probe_id: 9,
        intersection_source: RenderHybridGiTraceIntersectionSource::GlobalSdf,
        lighting_source: RenderHybridGiTraceLightingSource::ProbeLineage,
        intersection_backend_mask: 1 << 1,
        lighting_source_mask: 1 << 2,
        distance_bits: 3.5_f32.to_bits(),
        confidence_bits: 0.75_f32.to_bits(),
        fallback_reason: RenderHybridGiTraceFallbackReason::ScreenDataUnavailable,
        cost: RenderHybridGiTraceCostCounters {
            page_tests: 8,
            sdf_steps: 6,
            ..RenderHybridGiTraceCostCounters::default()
        },
    }]);

    let readback = HybridGiGpuReadback::new(
        vec![(5, 7)],
        vec![11, 12],
        vec![21],
        vec![(11, [1, 2, 3]), (12, [4, 5, 6])],
        vec![(11, [7, 8, 9])],
        Some(scene_prepare),
    )
    .with_radiance_cache_gpu_stage_dispatch_counts([1, 1, 1, 1, 1, 2]);

    let outputs = RenderHybridGiReadbackOutputs::from(readback);

    assert_eq!(
        outputs.cache_entries,
        vec![RenderHybridGiCacheEntryRecord { key: 5, value: 7 }]
    );
    assert_eq!(outputs.completed_probe_ids, vec![11, 12]);
    assert_eq!(outputs.completed_trace_region_ids, vec![21]);
    assert_eq!(outputs.probe_irradiance_rgb, vec![[1, 2, 3], [4, 5, 6]]);
    assert_eq!(outputs.probe_rt_lighting_rgb, vec![[7, 8, 9]]);
    assert_eq!(
        outputs.radiance_cache_gpu_stage_dispatch_counts,
        [1, 1, 1, 1, 1, 2]
    );
    assert_eq!(outputs.scene_prepare.occupied_atlas_slots, vec![1, 2]);
    assert_eq!(outputs.scene_prepare.occupied_capture_slots, vec![3]);
    assert_eq!(
        outputs.scene_prepare.atlas_samples,
        vec![RenderHybridGiScenePrepareSample {
            index: 1,
            rgba8: [10, 20, 30, 255],
        }]
    );
    assert_eq!(
        outputs.scene_prepare.capture_samples,
        vec![RenderHybridGiScenePrepareSample {
            index: 3,
            rgba8: [40, 50, 60, 255],
        }]
    );
    assert_eq!(outputs.scene_prepare.voxel_clipmap_ids, vec![9]);
    assert_eq!(
        outputs.scene_prepare.voxel_samples,
        vec![RenderHybridGiScenePrepareSample {
            index: 9,
            rgba8: [70, 80, 90, 255],
        }]
    );
    assert_eq!(outputs.scene_prepare.voxel_occupancy, vec![3]);
    assert_eq!(
        outputs.scene_prepare.voxel_occupancy_masks,
        vec![RenderHybridGiVoxelOccupancyMaskRecord {
            clipmap_id: 9,
            occupancy_mask: 0b1011,
        }]
    );
    assert_eq!(
        outputs.scene_prepare.voxel_cells,
        vec![RenderHybridGiVoxelCellRecord {
            clipmap_id: 9,
            cell_id: 2,
            occupancy: 4,
        }]
    );
    assert_eq!(
        outputs.scene_prepare.voxel_cell_samples,
        vec![RenderHybridGiVoxelCellSampleRecord {
            clipmap_id: 9,
            cell_id: 2,
            rgba8: [100, 110, 120, 255],
        }]
    );
    assert_eq!(
        outputs.scene_prepare.voxel_cell_dominant_nodes,
        vec![RenderHybridGiVoxelCellDominantNodeRecord {
            clipmap_id: 9,
            cell_id: 2,
            dominant_node_id: 77,
        }]
    );
    assert_eq!(
        outputs.scene_prepare.voxel_cell_dominant_samples,
        vec![RenderHybridGiVoxelCellSampleRecord {
            clipmap_id: 9,
            cell_id: 2,
            rgba8: [130, 140, 150, 255],
        }]
    );
    assert_eq!(
        outputs.scene_prepare.surface_cache_depth_samples,
        vec![RenderHybridGiScenePrepareSample {
            index: 1,
            rgba8: [96, 96, 96, 255],
        }]
    );
    assert_eq!(
        outputs.scene_prepare.probe_trace_tiles,
        vec![RenderHybridGiTraceTileRecord {
            tile_id: 0,
            probe_id: 9,
            trace_region_id: 2,
            ray_count: 32,
        }]
    );
    assert_eq!(outputs.scene_prepare.probe_trace_dispatch, [1, 1, 1]);
    assert_eq!(outputs.scene_prepare.probe_trace_diagnostics.len(), 1);
    assert_eq!(
        outputs.scene_prepare.probe_trace_diagnostics[0].intersection_source,
        RenderHybridGiTraceIntersectionSource::GlobalSdf
    );
    assert_eq!(
        outputs.scene_prepare.probe_trace_diagnostics[0]
            .cost
            .sdf_steps,
        6
    );
    assert_eq!(outputs.scene_prepare.texture_width, 64);
    assert_eq!(outputs.scene_prepare.texture_height, 32);
    assert_eq!(outputs.scene_prepare.texture_layers, 6);
}

#[test]
fn neutral_outputs_stay_empty_without_hybrid_gi_gpu_readback_payload() {
    let outputs = RenderHybridGiReadbackOutputs::from(HybridGiGpuReadback::default());

    assert_eq!(outputs, RenderHybridGiReadbackOutputs::default());
}
