use super::*;
use crate::core::framework::render::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiGlobalSdfStats, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord,
};

#[test]
fn gpu_completion_projects_neutral_hybrid_gi_readback_outputs() {
    let completion = HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
        cache_entries: vec![RenderHybridGiCacheEntryRecord { key: 17, value: 3 }],
        completed_probe_ids: vec![17, 19],
        completed_trace_region_ids: vec![5],
        probe_irradiance_rgb: vec![[16, 260, 64], [4, 8, 12]],
        probe_rt_lighting_rgb: vec![[1, 2, 3]],
        radiance_cache_gpu_stage_dispatch_counts: Default::default(),
        global_sdf_stats: None,
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            atlas_samples: vec![RenderHybridGiScenePrepareSample {
                index: 9,
                rgba8: [1, 2, 3, 4],
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
    })
    .expect("nonempty readback should create completion");

    assert_eq!(completion.cache_entries(), &[(17, 3)]);
    assert_eq!(completion.completed_probe_ids(), &[17, 19]);
    assert_eq!(completion.completed_trace_region_ids(), &[5]);
    assert_eq!(
        completion.probe_irradiance_rgb(),
        &[(17, [16, 255, 64]), (19, [4, 8, 12])]
    );
    assert_eq!(completion.probe_trace_lighting_rgb(), &[(17, [1, 2, 3])]);
    assert_eq!(
        completion.scene_prepare().unwrap().atlas_samples[0].rgba8,
        [1, 2, 3, 4]
    );
}

#[test]
fn gpu_completion_skips_empty_neutral_hybrid_gi_readback_outputs() {
    assert!(
        HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs::default())
            .is_none()
    );
}

#[test]
fn gpu_completion_keeps_gpu_authored_radiance_cache_dispatch_counts() {
    let completion = HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
        radiance_cache_gpu_stage_dispatch_counts: [1, 1, 1, 1, 1, 2],
        ..RenderHybridGiReadbackOutputs::default()
    })
    .expect("GPU-authored dispatch counts are runtime feedback");

    assert_eq!(
        completion.radiance_cache_gpu_stage_dispatch_counts(),
        [1, 1, 1, 1, 1, 2]
    );
}

#[test]
fn gpu_completion_keeps_global_sdf_runtime_stats() {
    let completion = HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
        global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
            cpu_prepare_time_us: 1500,
            cpu_mesh_object_collection_time_us: 200,
            cpu_mesh_scene_sync_time_us: 300,
            cpu_residency_time_us: 400,
            cpu_influence_update_time_us: 100,
            cpu_candidate_build_time_us: 500,
            mesh_projection_cache_hit: true,
            object_count: 17,
            resident_page_count: 9,
            dirty_page_count: 3,
            uploaded_page_count: 2,
            candidate_overflow_page_count: 1,
            candidate_contributor_count: 12,
            clipmap_fallback_count: 1,
            candidate_bucket_capacity_bytes: 256,
            persistent_resource_byte_count: 4096,
            transient_upload_byte_count: 256,
            ..RenderHybridGiGlobalSdfStats::default()
        }),
        ..RenderHybridGiReadbackOutputs::default()
    })
    .expect("Global SDF stats must be runtime feedback");

    let stats = completion
        .global_sdf_stats()
        .expect("Global SDF stats must survive completion projection");
    assert_eq!(stats.object_count, 17);
    assert_eq!(stats.cpu_influence_update_time_us, 100);
    assert!(stats.mesh_projection_cache_hit);
    assert_eq!(stats.uploaded_page_count, 2);
    assert_eq!(stats.candidate_overflow_page_count, 1);
    assert_eq!(stats.candidate_contributor_count, 12);
    assert_eq!(stats.clipmap_fallback_count, 1);
    assert_eq!(stats.candidate_bucket_capacity_bytes, 256);
    assert_eq!(stats.persistent_resource_byte_count, 4096);
}

#[test]
fn gpu_completion_skips_non_runtime_consumable_scene_prepare_metadata() {
    assert!(
        HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                occupied_atlas_slots: vec![3],
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        })
        .is_none()
    );
}

#[test]
fn gpu_completion_keeps_voxel_scene_prepare_readback_payload() {
    let completion = HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            voxel_occupancy_masks: vec![RenderHybridGiVoxelOccupancyMaskRecord {
                clipmap_id: 4,
                occupancy_mask: 0b1001,
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    })
    .expect("voxel readback payload should keep a completion");

    assert_eq!(
        completion.scene_prepare().unwrap().voxel_occupancy_masks[0].occupancy_mask,
        0b1001
    );
}

#[test]
fn gpu_completion_keeps_voxel_cell_scene_prepare_readback_payload() {
    let completion = HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            voxel_cell_dominant_samples: vec![RenderHybridGiVoxelCellSampleRecord {
                clipmap_id: 4,
                cell_id: 9,
                rgba8: [32, 48, 64, 255],
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    })
    .expect("voxel cell readback payload should keep a completion");

    assert_eq!(
        completion
            .scene_prepare()
            .unwrap()
            .voxel_cell_dominant_samples[0]
            .rgba8,
        [32, 48, 64, 255]
    );
}

#[test]
fn gpu_completion_preallocates_filtered_cache_projection() {
    let source = include_str!("../gpu_completion.rs");
    let capacity = concat!("Vec::with_capacity(", "cache_entry_records.len())");

    assert!(source.contains(capacity));
}

#[test]
fn gpu_completion_skips_cache_entries_outside_runtime_id_range() {
    let overflow = u64::from(u32::MAX) + 1;
    let completion = HybridGiGpuCompletion::from_readback_outputs(RenderHybridGiReadbackOutputs {
        cache_entries: vec![
            RenderHybridGiCacheEntryRecord { key: 17, value: 3 },
            RenderHybridGiCacheEntryRecord {
                key: overflow,
                value: 4,
            },
            RenderHybridGiCacheEntryRecord {
                key: 19,
                value: overflow,
            },
        ],
        ..RenderHybridGiReadbackOutputs::default()
    })
    .expect("valid cache entry should keep completion");

    assert_eq!(completion.cache_entries(), &[(17, 3)]);
}
