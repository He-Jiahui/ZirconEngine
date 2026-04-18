use zircon_framework::render::FrameHistoryHandle;

use crate::{runtime::ViewportFrameHistory, ViewportFrame};

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_history(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    frame: &ViewportFrame,
    allocated_history: Option<FrameHistoryHandle>,
) -> (Option<FrameHistoryHandle>, FrameHistoryHandle) {
    let previous_handle = record.history.as_ref().map(|history| history.handle);
    let history_handle = match (record.history.as_mut(), allocated_history) {
        (Some(history), None) => {
            history.update(
                frame.generation,
                context.compiled_pipeline.history_bindings.clone(),
                context.visibility_context.history_snapshot.clone(),
            );
            history.handle
        }
        (_, Some(handle)) => {
            record.history = Some(ViewportFrameHistory::new(
                handle,
                context.size,
                context.pipeline_handle,
                frame.generation,
                context.compiled_pipeline.history_bindings.clone(),
                context.visibility_context.history_snapshot.clone(),
            ));
            handle
        }
        (None, None) => unreachable!("rotation is required when no history exists"),
    };

    (previous_handle, history_handle)
}
