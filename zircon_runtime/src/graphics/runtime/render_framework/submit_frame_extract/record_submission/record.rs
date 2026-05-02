use crate::core::framework::render::FrameHistoryHandle;

use crate::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;
use super::super::submission_record_update::{
    HybridGiStatSnapshot, SubmissionRecordUpdate, VirtualGeometryStatSnapshot,
};
use super::record_capture::record_capture;
use super::record_history::record_history;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn record_submission(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    mut prepared: PreparedRuntimeSubmission,
    allocated_history: Option<FrameHistoryHandle>,
    frame: ViewportFrame,
    runtime_feedback: RuntimeFeedbackBatch,
) -> SubmissionRecordUpdate {
    let virtual_geometry_feedback = runtime_feedback
        .into_parts()
        .with_evictable_page_ids(prepared.take_virtual_geometry_evictable_page_ids());
    let virtual_geometry_indirect_segment_count = 0;
    let (previous_handle, history_handle) =
        record_history(record, context, &frame, allocated_history);
    record_capture(record, context, frame);
    let virtual_geometry_stats = update_virtual_geometry_runtime(
        record,
        virtual_geometry_feedback,
        virtual_geometry_indirect_segment_count,
    );

    SubmissionRecordUpdate::new(
        history_handle,
        previous_handle,
        HybridGiStatSnapshot::default(),
        virtual_geometry_stats,
    )
}

fn update_virtual_geometry_runtime(
    record: &mut ViewportRecord,
    feedback: crate::VirtualGeometryRuntimeFeedback,
    indirect_segment_count: usize,
) -> VirtualGeometryStatSnapshot {
    let Some(runtime) = record.virtual_geometry_runtime_mut() else {
        return VirtualGeometryStatSnapshot::default();
    };
    let update = runtime.update_after_render(feedback);
    let stats = update.stats();
    VirtualGeometryStatSnapshot::new(
        stats.page_table_entry_count(),
        stats.resident_page_count(),
        stats.pending_request_count(),
        stats.page_dependency_count(),
        stats.completed_page_count(),
        stats.replaced_page_count(),
        indirect_segment_count,
    )
}
