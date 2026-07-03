use super::*;
use crate::core::framework::scene::Mobility;

#[test]
fn prepared_queue_stats_count_gpu_morphed_sources_as_dynamic_geometry() {
    let stats = summarize_prepared_mesh_queue_items([
        item(
            profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::DynamicGpuMorphedSource,
                Mobility::Dynamic,
                false,
            ),
            true,
            false,
            "direct-gpu-morphed",
        ),
        gpu_skinned_item(
            skinned_gpu_profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource,
                Mobility::Dynamic,
                false,
            ),
            true,
            false,
            "gpu-skinned-morphed",
        ),
    ]);

    assert_eq!(stats.dynamic_geometry_draw_count, 2);
    assert_eq!(stats.gpu_morphed_source_draw_count, 1);
    assert_eq!(stats.gpu_skinned_morphed_source_draw_count, 1);
    assert_eq!(stats.prepared_geometry_draw_count, 0);
    assert_eq!(stats.skinned_draw_count, 1);
    assert_eq!(stats.skinned_palette_upload_count, 1);
    assert_eq!(stats.skinned_gpu_skinning_draw_count, 1);
}

#[test]
fn prepared_queue_stats_count_conventional_mesh_lod_draws() {
    let stats = summarize_prepared_mesh_queue_items([
        item(
            mesh_lod_profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Static,
                false,
            ),
            true,
            false,
            "lod-prepared",
        ),
        item(
            profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Static,
                false,
            ),
            true,
            false,
            "base-prepared",
        ),
    ]);

    assert_eq!(stats.lod_draw_count, 1);
    assert_eq!(stats.prepared_geometry_draw_count, 2);
    assert_eq!(stats.static_batch_candidate_group_count, 0);
}

#[test]
fn prepared_queue_stats_count_gpu_skinned_velocity_with_previous_palette() {
    let stats = summarize_prepared_mesh_queue_items([gpu_skinned_item(
        skinned_gpu_profile(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::Prepared,
            Mobility::Dynamic,
            false,
        ),
        true,
        true,
        "gpu-skinned-velocity",
    )]);

    assert_eq!(stats.skinned_previous_palette_upload_count, 1);
    assert_eq!(stats.skinned_gpu_velocity_draw_count, 1);
    assert_eq!(stats.previous_velocity_transform_draw_count, 1);
    assert_eq!(stats.missing_velocity_transform_draw_count, 0);
}

#[test]
fn prepared_queue_stats_exclude_gpu_skinned_draws_from_direct_batch_candidates() {
    let stats = summarize_prepared_mesh_queue_items([
        gpu_skinned_item(
            skinned_gpu_profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Dynamic,
                false,
            ),
            true,
            false,
            "gpu-skinned-shared-key",
        ),
        gpu_skinned_item(
            skinned_gpu_profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Dynamic,
                false,
            ),
            true,
            false,
            "gpu-skinned-shared-key",
        ),
    ]);

    assert_eq!(stats.prepared_geometry_draw_count, 2);
    assert_eq!(stats.skinned_gpu_skinning_draw_count, 2);
    assert_eq!(stats.dynamic_batch_candidate_group_count, 0);
    assert_eq!(stats.dynamic_batch_candidate_draw_count, 0);
    assert_eq!(stats.gpu_instancing_candidate_group_count, 0);
    assert_eq!(stats.gpu_instancing_candidate_draw_count, 0);
}
