use crate::core::framework::render::FrameHistoryHandle;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::record::particle_feedback_stat_snapshot;
use super::record::update_hybrid_gi_runtime;
use super::record::update_virtual_geometry_runtime;
use super::record_history::record_history;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn record_present_submission(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    mut prepared: PreparedRuntimeSubmission,
    allocated_history: Option<FrameHistoryHandle>,
    generation: u64,
    runtime_feedback: RuntimeFeedbackBatch,
) -> SubmissionRecordUpdate {
    let (hybrid_gi_feedback, particle_feedback, virtual_geometry_feedback) =
        runtime_feedback.into_parts();
    let hybrid_gi_feedback =
        hybrid_gi_feedback.with_evictable_probe_ids(prepared.take_hybrid_gi_evictable_probe_ids());
    let virtual_geometry_feedback = virtual_geometry_feedback
        .with_evictable_page_ids(prepared.take_virtual_geometry_evictable_page_ids());
    let virtual_geometry_indirect_segment_count = 0;
    let (previous_handle, history_handle) =
        record_history(record, context, generation, allocated_history);
    record.store_presented_pipeline(context.compiled_pipeline().clone());
    let hybrid_gi_stats = update_hybrid_gi_runtime(record, hybrid_gi_feedback);
    let particle_stats = particle_feedback_stat_snapshot(particle_feedback);
    let virtual_geometry_stats = update_virtual_geometry_runtime(
        record,
        virtual_geometry_feedback,
        virtual_geometry_indirect_segment_count,
    );

    SubmissionRecordUpdate::new(
        history_handle,
        previous_handle,
        hybrid_gi_stats,
        particle_stats,
        virtual_geometry_stats,
    )
}
