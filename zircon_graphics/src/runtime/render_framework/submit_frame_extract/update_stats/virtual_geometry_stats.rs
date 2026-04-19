use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;

pub(super) fn update_virtual_geometry_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
) {
    state.stats.last_virtual_geometry_visible_cluster_count = context
        .visibility_context
        .virtual_geometry_visible_clusters
        .len();
    state.stats.last_virtual_geometry_requested_page_count = context
        .visibility_context
        .virtual_geometry_page_upload_plan
        .requested_pages
        .len();
    state.stats.last_virtual_geometry_dirty_page_count = context
        .visibility_context
        .virtual_geometry_page_upload_plan
        .dirty_requested_pages
        .len();
    state.stats.last_virtual_geometry_page_table_entry_count =
        record_update.virtual_geometry_stats.page_table_entry_count;
    state.stats.last_virtual_geometry_resident_page_count =
        record_update.virtual_geometry_stats.resident_page_count;
    state.stats.last_virtual_geometry_pending_request_count =
        record_update.virtual_geometry_stats.pending_request_count;
    state.stats.last_virtual_geometry_completed_page_count =
        record_update.virtual_geometry_stats.completed_page_count;
    state.stats.last_virtual_geometry_replaced_page_count =
        record_update.virtual_geometry_stats.replaced_page_count;
    state.stats.last_virtual_geometry_indirect_draw_count =
        state.renderer.last_virtual_geometry_indirect_draw_count() as usize;
    state.stats.last_virtual_geometry_indirect_buffer_count =
        state.renderer.last_virtual_geometry_indirect_buffer_count() as usize;
    state.stats.last_virtual_geometry_indirect_args_count =
        state.renderer.last_virtual_geometry_indirect_args_count() as usize;
    state.stats.last_virtual_geometry_indirect_segment_count = state
        .renderer
        .last_virtual_geometry_indirect_segment_count()
        as usize;
}

pub(super) fn reset_virtual_geometry_stats(state: &mut RenderFrameworkState) {
    state.stats.last_virtual_geometry_visible_cluster_count = 0;
    state.stats.last_virtual_geometry_requested_page_count = 0;
    state.stats.last_virtual_geometry_dirty_page_count = 0;
    state.stats.last_virtual_geometry_page_table_entry_count = 0;
    state.stats.last_virtual_geometry_resident_page_count = 0;
    state.stats.last_virtual_geometry_pending_request_count = 0;
    state.stats.last_virtual_geometry_completed_page_count = 0;
    state.stats.last_virtual_geometry_replaced_page_count = 0;
    state.stats.last_virtual_geometry_indirect_draw_count = 0;
    state.stats.last_virtual_geometry_indirect_buffer_count = 0;
    state.stats.last_virtual_geometry_indirect_args_count = 0;
    state.stats.last_virtual_geometry_indirect_segment_count = 0;
}
