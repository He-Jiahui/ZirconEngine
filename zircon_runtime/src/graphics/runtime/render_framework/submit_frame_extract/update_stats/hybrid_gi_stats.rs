use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use crate::core::framework::render::RenderHybridGiPayloadSource;

pub(super) fn update_hybrid_gi_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
) {
    let hybrid_gi_stats = record_update.hybrid_gi_stats();
    state.stats.last_hybrid_gi_active_probe_count =
        context.visibility_context().hybrid_gi_active_probes.len();
    state.stats.last_hybrid_gi_requested_probe_count = context
        .hybrid_gi_update_plan()
        .map(|plan| plan.requested_probe_ids.len())
        .unwrap_or(0);
    state.stats.last_hybrid_gi_dirty_probe_count = context
        .hybrid_gi_update_plan()
        .map(|plan| plan.dirty_requested_probe_ids.len())
        .unwrap_or(0);
    state.stats.last_hybrid_gi_cache_entry_count = hybrid_gi_stats.cache_entry_count();
    state.stats.last_hybrid_gi_resident_probe_count = hybrid_gi_stats.resident_probe_count();
    state.stats.last_hybrid_gi_pending_update_count = hybrid_gi_stats.pending_update_count();
    state.stats.last_hybrid_gi_scheduled_trace_region_count =
        hybrid_gi_stats.scheduled_trace_region_count();
    state.stats.last_hybrid_gi_scene_card_count = hybrid_gi_stats.scene_card_count();
    state.stats.last_hybrid_gi_scene_screen_probe_count =
        hybrid_gi_stats.scene_screen_probe_count();
    state.stats.last_hybrid_gi_scene_radiance_cache_entry_count =
        hybrid_gi_stats.scene_radiance_cache_entry_count();
    state
        .stats
        .last_hybrid_gi_radiance_cache_resident_probe_count =
        hybrid_gi_stats.radiance_cache_resident_probe_count();
    state.stats.last_hybrid_gi_radiance_cache_update_probe_count =
        hybrid_gi_stats.radiance_cache_update_probe_count();
    state
        .stats
        .last_hybrid_gi_radiance_cache_truncated_demand_count =
        hybrid_gi_stats.radiance_cache_truncated_demand_count();
    state.stats.last_hybrid_gi_radiance_cache_generation =
        hybrid_gi_stats.radiance_cache_generation();
    state.stats.last_hybrid_gi_radiance_cache_scroll_count =
        hybrid_gi_stats.radiance_cache_scroll_count();
    state
        .stats
        .last_hybrid_gi_radiance_cache_history_clear_count =
        hybrid_gi_stats.radiance_cache_history_clear_count();
    state
        .stats
        .last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts =
        hybrid_gi_stats.radiance_cache_gpu_stage_dispatch_counts();
    state.stats.last_hybrid_gi_surface_cache_resident_page_count =
        hybrid_gi_stats.surface_cache_resident_page_count();
    state.stats.last_hybrid_gi_surface_cache_dirty_page_count =
        hybrid_gi_stats.surface_cache_dirty_page_count();
    state.stats.last_hybrid_gi_surface_cache_feedback_card_count =
        hybrid_gi_stats.surface_cache_feedback_card_count();
    state.stats.last_hybrid_gi_surface_cache_capture_slot_count =
        hybrid_gi_stats.surface_cache_capture_slot_count();
    state
        .stats
        .last_hybrid_gi_surface_cache_invalidated_page_count =
        hybrid_gi_stats.surface_cache_invalidated_page_count();
    state.stats.last_hybrid_gi_surface_cache_depth_sample_count =
        hybrid_gi_stats.surface_cache_depth_sample_count();
    state.stats.last_hybrid_gi_probe_trace_tile_count = hybrid_gi_stats.probe_trace_tile_count();
    state.stats.last_hybrid_gi_probe_trace_dispatch_group_count =
        hybrid_gi_stats.probe_trace_dispatch_group_count();
    state.stats.last_hybrid_gi_voxel_resident_clipmap_count =
        hybrid_gi_stats.voxel_resident_clipmap_count();
    state.stats.last_hybrid_gi_voxel_dirty_clipmap_count =
        hybrid_gi_stats.voxel_dirty_clipmap_count();
    state.stats.last_hybrid_gi_voxel_invalidated_clipmap_count =
        hybrid_gi_stats.voxel_invalidated_clipmap_count();
    let global_sdf_stats = hybrid_gi_stats.global_sdf_stats();
    state.stats.last_hybrid_gi_global_sdf_cpu_prepare_time_us =
        global_sdf_stats.cpu_prepare_time_us;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us =
        global_sdf_stats.cpu_mesh_object_collection_time_us;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us =
        global_sdf_stats.cpu_mesh_scene_sync_time_us;
    state.stats.last_hybrid_gi_global_sdf_cpu_residency_time_us =
        global_sdf_stats.cpu_residency_time_us;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_influence_update_time_us =
        global_sdf_stats.cpu_influence_update_time_us;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_candidate_build_time_us =
        global_sdf_stats.cpu_candidate_build_time_us;
    state
        .stats
        .last_hybrid_gi_global_sdf_mesh_projection_cache_hit =
        global_sdf_stats.mesh_projection_cache_hit;
    state.stats.last_hybrid_gi_global_sdf_object_count = global_sdf_stats.object_count;
    state.stats.last_hybrid_gi_global_sdf_resident_page_count =
        global_sdf_stats.resident_page_count;
    state.stats.last_hybrid_gi_global_sdf_sampleable_page_count =
        global_sdf_stats.sampleable_page_count;
    state.stats.last_hybrid_gi_global_sdf_dirty_page_count = global_sdf_stats.dirty_page_count;
    state.stats.last_hybrid_gi_global_sdf_dispatched_page_count =
        global_sdf_stats.dispatched_page_count;
    state.stats.last_hybrid_gi_global_sdf_uploaded_page_count =
        global_sdf_stats.uploaded_page_count;
    state.stats.last_hybrid_gi_global_sdf_deferred_page_count =
        global_sdf_stats.deferred_page_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_candidate_overflow_page_count =
        global_sdf_stats.candidate_overflow_page_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_candidate_contributor_count =
        global_sdf_stats.candidate_contributor_count;
    state.stats.last_hybrid_gi_global_sdf_clipmap_fallback_count =
        global_sdf_stats.clipmap_fallback_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_candidate_bucket_capacity_bytes =
        global_sdf_stats.candidate_bucket_capacity_bytes;
    state
        .stats
        .last_hybrid_gi_global_sdf_persistent_resource_byte_count =
        global_sdf_stats.persistent_resource_byte_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_buffer_creation_count =
        global_sdf_stats.transient_buffer_creation_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_bind_group_creation_count =
        global_sdf_stats.transient_bind_group_creation_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_parameter_upload_byte_count =
        global_sdf_stats.transient_parameter_upload_byte_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_page_upload_byte_count =
        global_sdf_stats.transient_page_upload_byte_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_mesh_upload_byte_count =
        global_sdf_stats.transient_mesh_upload_byte_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_completion_upload_byte_count =
        global_sdf_stats.transient_completion_upload_byte_count;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_upload_byte_count =
        global_sdf_stats.transient_upload_byte_count;
    state.stats.last_hybrid_gi_payload_source = context.hybrid_gi_payload_source();
    state.stats.last_hybrid_gi_resolved_settings = hybrid_gi_stats.resolved_settings();
}

pub(super) fn reset_hybrid_gi_stats(state: &mut RenderFrameworkState) {
    state.stats.last_hybrid_gi_active_probe_count = 0;
    state.stats.last_hybrid_gi_requested_probe_count = 0;
    state.stats.last_hybrid_gi_dirty_probe_count = 0;
    state.stats.last_hybrid_gi_cache_entry_count = 0;
    state.stats.last_hybrid_gi_resident_probe_count = 0;
    state.stats.last_hybrid_gi_pending_update_count = 0;
    state.stats.last_hybrid_gi_scheduled_trace_region_count = 0;
    state.stats.last_hybrid_gi_scene_card_count = 0;
    state.stats.last_hybrid_gi_scene_screen_probe_count = 0;
    state.stats.last_hybrid_gi_scene_radiance_cache_entry_count = 0;
    state
        .stats
        .last_hybrid_gi_radiance_cache_resident_probe_count = 0;
    state.stats.last_hybrid_gi_radiance_cache_update_probe_count = 0;
    state
        .stats
        .last_hybrid_gi_radiance_cache_truncated_demand_count = 0;
    state.stats.last_hybrid_gi_radiance_cache_generation = 0;
    state.stats.last_hybrid_gi_radiance_cache_scroll_count = 0;
    state
        .stats
        .last_hybrid_gi_radiance_cache_history_clear_count = 0;
    state
        .stats
        .last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts = Default::default();
    state.stats.last_hybrid_gi_surface_cache_resident_page_count = 0;
    state.stats.last_hybrid_gi_surface_cache_dirty_page_count = 0;
    state.stats.last_hybrid_gi_surface_cache_feedback_card_count = 0;
    state.stats.last_hybrid_gi_surface_cache_capture_slot_count = 0;
    state
        .stats
        .last_hybrid_gi_surface_cache_invalidated_page_count = 0;
    state.stats.last_hybrid_gi_surface_cache_depth_sample_count = 0;
    state.stats.last_hybrid_gi_probe_trace_tile_count = 0;
    state.stats.last_hybrid_gi_probe_trace_dispatch_group_count = [0; 3];
    state.stats.last_hybrid_gi_voxel_resident_clipmap_count = 0;
    state.stats.last_hybrid_gi_voxel_dirty_clipmap_count = 0;
    state.stats.last_hybrid_gi_voxel_invalidated_clipmap_count = 0;
    state.stats.last_hybrid_gi_global_sdf_cpu_prepare_time_us = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us = 0;
    state.stats.last_hybrid_gi_global_sdf_cpu_residency_time_us = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_influence_update_time_us = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_cpu_candidate_build_time_us = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_mesh_projection_cache_hit = false;
    state.stats.last_hybrid_gi_global_sdf_object_count = 0;
    state.stats.last_hybrid_gi_global_sdf_resident_page_count = 0;
    state.stats.last_hybrid_gi_global_sdf_sampleable_page_count = 0;
    state.stats.last_hybrid_gi_global_sdf_dirty_page_count = 0;
    state.stats.last_hybrid_gi_global_sdf_dispatched_page_count = 0;
    state.stats.last_hybrid_gi_global_sdf_uploaded_page_count = 0;
    state.stats.last_hybrid_gi_global_sdf_deferred_page_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_candidate_overflow_page_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_candidate_contributor_count = 0;
    state.stats.last_hybrid_gi_global_sdf_clipmap_fallback_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_candidate_bucket_capacity_bytes = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_persistent_resource_byte_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_buffer_creation_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_bind_group_creation_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_parameter_upload_byte_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_page_upload_byte_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_mesh_upload_byte_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_completion_upload_byte_count = 0;
    state
        .stats
        .last_hybrid_gi_global_sdf_transient_upload_byte_count = 0;
    state.stats.last_hybrid_gi_payload_source = RenderHybridGiPayloadSource::None;
    state.stats.last_hybrid_gi_resolved_settings = None;
}
