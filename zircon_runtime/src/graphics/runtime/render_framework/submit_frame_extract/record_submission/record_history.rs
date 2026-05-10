use crate::core::framework::render::FrameHistoryHandle;

use crate::runtime::ViewportFrameHistory;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_history(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    generation: u64,
    allocated_history: Option<FrameHistoryHandle>,
) -> (Option<FrameHistoryHandle>, FrameHistoryHandle) {
    let previous_handle = record.history().map(|history| history.handle());
    let history_handle = match (record.history_mut(), allocated_history) {
        (Some(history), None) => {
            history.update(
                generation,
                context.compiled_pipeline().history_bindings.clone(),
                context.visibility_context().history_snapshot.clone(),
                context.history_validation_key().clone(),
            );
            history.handle()
        }
        (_, Some(handle)) => {
            record.replace_history(ViewportFrameHistory::new(
                handle,
                context.size(),
                context.pipeline_handle(),
                generation,
                context.compiled_pipeline().history_bindings.clone(),
                context.visibility_context().history_snapshot.clone(),
                context.history_validation_key().clone(),
            ));
            handle
        }
        (None, None) => unreachable!("rotation is required when no history exists"),
    };

    (previous_handle, history_handle)
}
