use crate::core::framework::render::{FrameHistoryHandle, RenderViewportHandle};

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::record::particle_feedback_stat_snapshot;
use super::record::update_hybrid_gi_runtime;
use super::record::update_virtual_geometry_runtime;
use super::record_history::record_history;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn record_present_submission(
    record: &mut ViewportRecord,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
    allocated_history: Option<FrameHistoryHandle>,
    generation: u64,
    runtime_feedback: RuntimeFeedbackBatch,
) -> SubmissionRecordUpdate {
    record.store_visible_spatial_query(
        viewport,
        context.source_world(),
        generation,
        context.visibility_context(),
    );
    let (hybrid_gi_feedback, particle_feedback, virtual_geometry_feedback) =
        runtime_feedback.into_parts();
    let (previous_handle, history_handle, history_status) =
        record_history(record, context, generation, allocated_history);
    record.store_presented_pipeline(context.compiled_pipeline_shared());
    let hybrid_gi_stats =
        update_hybrid_gi_runtime(record, context.camera_history_key(), hybrid_gi_feedback);
    let particle_stats = particle_feedback_stat_snapshot(particle_feedback);
    let virtual_geometry_stats = update_virtual_geometry_runtime(
        record,
        context.camera_history_key(),
        virtual_geometry_feedback,
    );

    SubmissionRecordUpdate::new(
        history_handle,
        previous_handle,
        history_status,
        crate::core::framework::render::RenderCaptureReport::not_captured(
            context.output_target().kind(),
        ),
        hybrid_gi_stats,
        particle_stats,
        virtual_geometry_stats,
    )
}
