use crate::graphics::scene::scene_renderer::core::ViewportAsyncCaptureSubmission;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_capture(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    frame: ViewportAsyncCaptureSubmission,
) {
    if !frame.capture_admitted {
        return;
    }
    let compiled_pipeline = context.compiled_pipeline_shared();
    record.register_async_capture(
        frame.generation,
        frame.capture_size,
        frame.capture_report,
        compiled_pipeline,
    );
}
