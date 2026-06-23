use super::super::build_mesh_draws::{
    PendingMeshCommandCacheExtractionStats, PendingMeshCommandCachePlanStats,
};
use super::super::mesh_pass::{MeshDrawReplayStats, MeshPassCommandBufferStats};
use super::*;
use crate::core::framework::render::{
    RenderGpuSceneUploadPath, RenderVirtualGeometryExecutionDraw,
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
};
use crate::graphics::scene::gpu_scene::{GpuSceneStats, GpuSceneUploadPath, GpuSceneUploadReport};

#[test]
fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts() {
    let stats = PreparedMeshQueueStats::default().with_mesh_pass_command_buffer_stats(
        MeshPassCommandBufferStats {
            command_count: 9,
            depth_prepass_command_count: 2,
            shadow_command_count: 1,
            opaque_command_count: 3,
            alpha_mask_command_count: 1,
            transparent_command_count: 1,
            velocity_command_count: 1,
            taa_reactive_mask_command_count: 1,
            direct_indexed_count: 7,
            indirect_indexed_count: 2,
            gpu_scene_instance_count: 9,
            cached_command_hit_count: 0,
            command_rebuild_count: 9,
            dynamic_command_count: 9,
            cache_miss_count: 3,
            cache_invalidated_transform_count: 0,
            cache_invalidated_geometry_count: 1,
            cache_invalidated_material_count: 2,
            indirect_batch_count: 2,
            indirect_batched_draw_count: 5,
            indirect_fallback_draw_count: 4,
            indirect_args_count: 5,
        },
    );

    assert_eq!(stats.command_count, 9);
    assert_eq!(stats.depth_prepass_command_count, 2);
    assert_eq!(stats.shadow_command_count, 1);
    assert_eq!(stats.opaque_command_count, 3);
    assert_eq!(stats.alpha_mask_command_count, 1);
    assert_eq!(stats.transparent_command_count, 1);
    assert_eq!(stats.velocity_command_count, 1);
    assert_eq!(stats.taa_reactive_mask_command_count, 1);
    assert_eq!(stats.cached_command_hit_count, 0);
    assert_eq!(stats.command_rebuild_count, 9);
    assert_eq!(stats.dynamic_command_count, 9);
    assert_eq!(stats.cache_miss_count, 3);
    assert_eq!(stats.cache_invalidated_transform_count, 0);
    assert_eq!(stats.cache_invalidated_geometry_count, 1);
    assert_eq!(stats.cache_invalidated_material_count, 2);
    assert_eq!(stats.indirect_batch_count, 2);
    assert_eq!(stats.indirect_batched_draw_count, 5);
    assert_eq!(stats.indirect_fallback_draw_count, 4);
    assert_eq!(stats.indirect_args_count, 5);
    assert_eq!(stats.state_change_count, 0);
    assert_eq!(stats.bind_skip_count, 0);
}

#[test]
fn prepared_queue_stats_carry_virtual_geometry_indirect_counts() {
    let stats = PreparedMeshQueueStats::default().with_virtual_geometry_indirect_stats(
        PreparedMeshVirtualGeometryIndirectStats {
            draw_count: 3,
            buffer_count: 5,
            args_count: 3,
            segment_count: 2,
        },
    );

    assert_eq!(stats.virtual_geometry_indirect_draw_count, 3);
    assert_eq!(stats.virtual_geometry_indirect_buffer_count, 5);
    assert_eq!(stats.virtual_geometry_indirect_args_count, 3);
    assert_eq!(stats.virtual_geometry_indirect_segment_count, 2);
}

#[test]
fn prepared_queue_stats_carry_virtual_geometry_execution_counts() {
    let execution_stats = PreparedMeshVirtualGeometryExecutionStats::from_execution_draws([
        execution_draw(0, 10, RenderVirtualGeometryExecutionState::Resident),
        execution_draw(1, 10, RenderVirtualGeometryExecutionState::Resident),
        execution_draw(2, 11, RenderVirtualGeometryExecutionState::PendingUpload),
        non_virtual_geometry_indirect_draw(),
    ]);
    let stats =
        PreparedMeshQueueStats::default().with_virtual_geometry_execution_stats(execution_stats);

    assert_eq!(stats.virtual_geometry_execution_draw_count, 3);
    assert_eq!(stats.virtual_geometry_execution_segment_count, 2);
    assert_eq!(stats.virtual_geometry_execution_page_count, 2);
    assert_eq!(stats.virtual_geometry_execution_resident_segment_count, 1);
    assert_eq!(stats.virtual_geometry_execution_pending_segment_count, 1);
    assert_eq!(stats.virtual_geometry_execution_missing_segment_count, 0);
    assert_eq!(stats.virtual_geometry_execution_repeated_draw_count, 1);
}

#[test]
fn prepared_queue_stats_carry_pending_command_cache_plan_counts() {
    let stats = PreparedMeshQueueStats::default().with_pending_command_cache_plan_stats(
        PendingMeshCommandCachePlanStats {
            static_command_cache_draw_candidate_count: 3,
            static_command_cache_phase_candidate_count: 7,
            static_command_cache_depth_prepass_candidate_count: 3,
            static_command_cache_shadow_candidate_count: 2,
            static_command_cache_opaque_candidate_count: 1,
            static_command_cache_alpha_mask_candidate_count: 1,
        },
    );

    assert_eq!(stats.pending_static_command_cache_draw_candidate_count, 3);
    assert_eq!(stats.pending_static_command_cache_phase_candidate_count, 7);
    assert_eq!(
        stats.pending_static_command_cache_depth_prepass_candidate_count,
        3
    );
    assert_eq!(stats.pending_static_command_cache_shadow_candidate_count, 2);
    assert_eq!(stats.pending_static_command_cache_opaque_candidate_count, 1);
    assert_eq!(
        stats.pending_static_command_cache_alpha_mask_candidate_count,
        1
    );
}

#[test]
fn prepared_queue_stats_carry_pending_command_cache_extraction_counts() {
    let stats = PreparedMeshQueueStats::default().with_pending_command_cache_extraction_stats(
        PendingMeshCommandCacheExtractionStats {
            skipped_mesh_draw_count: 2,
            skipped_phase_count: 5,
            visibility_pruned_mesh_draw_count: 1,
            residual_material_phase_draw_count: 3,
            residual_rebuild_input_missing_draw_count: 4,
            residual_rebuild_rejected_draw_count: 6,
        },
    );

    assert_eq!(
        stats.pre_mesh_draw_static_command_cache_skipped_draw_count,
        2
    );
    assert_eq!(
        stats.pre_mesh_draw_static_command_cache_skipped_phase_count,
        5
    );
    assert_eq!(
        stats.pre_mesh_draw_static_command_cache_visibility_pruned_draw_count,
        1
    );
    assert_eq!(
        stats.pre_mesh_draw_static_command_cache_residual_material_phase_draw_count,
        3
    );
    assert_eq!(
        stats.pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count,
        4
    );
    assert_eq!(
        stats.pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count,
        6
    );
}

#[test]
fn prepared_queue_stats_carry_mesh_draw_replay_counts() {
    let stats =
        PreparedMeshQueueStats::default().with_mesh_draw_replay_stats(MeshDrawReplayStats {
            draw_call_count: 12,
            state_change_count: 4,
            bind_skip_count: 7,
        });

    assert_eq!(stats.state_change_count, 4);
    assert_eq!(stats.bind_skip_count, 7);
}

#[test]
fn prepared_queue_stats_carry_gpu_scene_counts() {
    let stats = PreparedMeshQueueStats::default().with_gpu_scene_stats(
        GpuSceneStats {
            primitive_count: 5,
            instance_count: 7,
            light_count: 11,
            dirty_entry_count: 3,
            uploaded_bytes: 99,
            primitive_capacity: 1024,
            instance_capacity: 2048,
            light_capacity: 256,
            free_span_count: 2,
        },
        GpuSceneUploadReport {
            upload_path: GpuSceneUploadPath::DirectQueueWrite,
            uploaded_bytes: 88,
            primitive_upload_range_count: 4,
            instance_upload_range_count: 6,
            ..GpuSceneUploadReport::default()
        },
    );

    assert_eq!(stats.gpu_scene_primitive_count, 5);
    assert_eq!(stats.gpu_scene_instance_count, 7);
    assert_eq!(stats.gpu_scene_dirty_entry_count, 3);
    assert_eq!(stats.gpu_scene_uploaded_bytes, 88);
    assert_eq!(
        stats.gpu_scene_upload_path,
        RenderGpuSceneUploadPath::DirectQueueWrite
    );
    assert_eq!(stats.gpu_scene_free_span_count, 2);
    assert_eq!(stats.gpu_scene_primitive_upload_range_count, 4);
    assert_eq!(stats.gpu_scene_instance_upload_range_count, 6);
}

fn execution_draw(
    draw_ref_index: u32,
    page_id: u32,
    state: RenderVirtualGeometryExecutionState,
) -> RenderVirtualGeometryExecutionDraw {
    RenderVirtualGeometryExecutionDraw {
        indirect_args_buffer_available: true,
        indirect_args_offset: u64::from(draw_ref_index) * 20,
        uses_indirect_draw: true,
        execution_selection_key: Some((42, page_id)),
        execution_segment: RenderVirtualGeometryExecutionSegment {
            original_index: draw_ref_index,
            instance_index: Some(1),
            entity: 42,
            page_id,
            draw_ref_index,
            submission_index: Some(page_id),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            submission_slot: Some(page_id),
            state,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
        },
        submission_order_record: Some((Some(1), 42, page_id)),
        draw_submission_record: Some((42, page_id, draw_ref_index, draw_ref_index as usize)),
        draw_submission_token_record: Some((42, page_id, page_id, 0, draw_ref_index as usize)),
        execution_draw_ref_index: draw_ref_index,
    }
}

fn non_virtual_geometry_indirect_draw() -> RenderVirtualGeometryExecutionDraw {
    let mut draw = execution_draw(99, 99, RenderVirtualGeometryExecutionState::Resident);
    draw.execution_selection_key = None;
    draw.execution_segment.entity = 0;
    draw
}
