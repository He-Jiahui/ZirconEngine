use crate::core::framework::render::CapturedFrame;

use crate::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_capture(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    frame: ViewportFrame,
) {
    record.store_capture(
        context.compiled_pipeline().clone(),
        CapturedFrame::new(frame.width, frame.height, frame.rgba, frame.generation),
    );
}
