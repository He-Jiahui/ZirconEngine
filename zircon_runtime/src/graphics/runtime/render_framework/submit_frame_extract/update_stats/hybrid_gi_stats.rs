use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;

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
    state.stats.last_hybrid_gi_voxel_resident_clipmap_count =
        hybrid_gi_stats.voxel_resident_clipmap_count();
    state.stats.last_hybrid_gi_voxel_dirty_clipmap_count =
        hybrid_gi_stats.voxel_dirty_clipmap_count();
    state.stats.last_hybrid_gi_voxel_invalidated_clipmap_count =
        hybrid_gi_stats.voxel_invalidated_clipmap_count();
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
    state.stats.last_hybrid_gi_surface_cache_resident_page_count = 0;
    state.stats.last_hybrid_gi_surface_cache_dirty_page_count = 0;
    state.stats.last_hybrid_gi_surface_cache_feedback_card_count = 0;
    state.stats.last_hybrid_gi_surface_cache_capture_slot_count = 0;
    state
        .stats
        .last_hybrid_gi_surface_cache_invalidated_page_count = 0;
    state.stats.last_hybrid_gi_voxel_resident_clipmap_count = 0;
    state.stats.last_hybrid_gi_voxel_dirty_clipmap_count = 0;
    state.stats.last_hybrid_gi_voxel_invalidated_clipmap_count = 0;
}
