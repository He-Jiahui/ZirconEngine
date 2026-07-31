use crate::core::framework::render::CapturedFrame;

use crate::graphics::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_capture(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    frame: ViewportFrame,
) {
    let compiled_pipeline = context.compiled_pipeline_shared();
    let graph_dump = Some(record.capture_graph_dump(&compiled_pipeline));
    record.store_capture(
        compiled_pipeline,
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
