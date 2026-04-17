use super::super::super::render_server_state::RenderServerState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;

pub(super) fn update_hybrid_gi_stats(
    state: &mut RenderServerState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
) {
    state.stats.last_hybrid_gi_active_probe_count =
        context.visibility_context.hybrid_gi_active_probes.len();
    state.stats.last_hybrid_gi_requested_probe_count = context
        .visibility_context
        .hybrid_gi_update_plan
        .requested_probe_ids
        .len();
    state.stats.last_hybrid_gi_dirty_probe_count = context
        .visibility_context
        .hybrid_gi_update_plan
        .dirty_requested_probe_ids
        .len();
    state.stats.last_hybrid_gi_cache_entry_count = record_update.hybrid_gi_stats.cache_entry_count;
    state.stats.last_hybrid_gi_resident_probe_count =
        record_update.hybrid_gi_stats.resident_probe_count;
    state.stats.last_hybrid_gi_pending_update_count =
        record_update.hybrid_gi_stats.pending_update_count;
    state.stats.last_hybrid_gi_scheduled_trace_region_count =
        record_update.hybrid_gi_stats.scheduled_trace_region_count;
}

pub(super) fn reset_hybrid_gi_stats(state: &mut RenderServerState) {
    state.stats.last_hybrid_gi_active_probe_count = 0;
    state.stats.last_hybrid_gi_requested_probe_count = 0;
    state.stats.last_hybrid_gi_dirty_probe_count = 0;
    state.stats.last_hybrid_gi_cache_entry_count = 0;
    state.stats.last_hybrid_gi_resident_probe_count = 0;
    state.stats.last_hybrid_gi_pending_update_count = 0;
    state.stats.last_hybrid_gi_scheduled_trace_region_count = 0;
}
