use crate::core::framework::render::CapturedFrame;

use crate::graphics::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_capture(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    frame: ViewportFrame,
) {
    let graph_dump = Some(context.compiled_pipeline().graph.dump().to_text());
    record.store_capture(
        context.compiled_pipeline().clone(),
        CapturedFrame::with_capture_report_and_graph_dump(
            frame.width,
            frame.height,
            frame.rgba,
            frame.generation,
            frame.capture_report,
            graph_dump,
        ),
    );
}
