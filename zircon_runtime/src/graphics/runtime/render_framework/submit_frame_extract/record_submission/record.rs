use crate::core::framework::render::FrameHistoryHandle;

use crate::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::record_capture::record_capture;
use super::record_history::record_history;
use super::update_hybrid_gi_runtime::update_hybrid_gi_runtime;
use super::update_virtual_geometry_runtime::update_virtual_geometry_runtime;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn record_submission(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    prepared: PreparedRuntimeSubmission,
    allocated_history: Option<FrameHistoryHandle>,
    frame: ViewportFrame,
    runtime_feedback: RuntimeFeedbackBatch,
) -> SubmissionRecordUpdate {
    let mut prepared = prepared;
    let (hybrid_gi_feedback, virtual_geometry_feedback) = runtime_feedback.into_parts();
    let hybrid_gi_feedback =
        hybrid_gi_feedback.with_evictable_probe_ids(prepared.take_hybrid_gi_evictable_probe_ids());
    let virtual_geometry_feedback = virtual_geometry_feedback
        .with_evictable_page_ids(prepared.take_virtual_geometry_evictable_page_ids());
    let (previous_handle, history_handle) =
        record_history(record, context, &frame, allocated_history);
    record_capture(record, context, frame);
    let hybrid_gi_stats = update_hybrid_gi_runtime(&mut prepared, &hybrid_gi_feedback);
    let virtual_geometry_stats =
        update_virtual_geometry_runtime(context, &mut prepared, &virtual_geometry_feedback);

    let (hybrid_gi_runtime, virtual_geometry_runtime) = prepared.into_runtime_states();
    record.replace_runtime_states(hybrid_gi_runtime, virtual_geometry_runtime);

    SubmissionRecordUpdate::new(
        history_handle,
        previous_handle,
        hybrid_gi_stats,
        virtual_geometry_stats,
    )
}
