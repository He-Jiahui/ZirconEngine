use zircon_framework::render::CapturedFrame;

use crate::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_capture(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    frame: ViewportFrame,
) {
    record.compiled_pipeline = Some(context.compiled_pipeline.clone());
    record.last_capture = Some(CapturedFrame::new(
        frame.width,
        frame.height,
        frame.rgba,
        frame.generation,
    ));
}
