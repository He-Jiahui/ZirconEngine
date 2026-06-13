use crate::core::framework::render::{FrameHistoryHandle, FrameHistoryStatus};

use crate::runtime::ViewportFrameHistory;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn record_history(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    generation: u64,
    allocated_history: Option<FrameHistoryHandle>,
) -> (
    Option<FrameHistoryHandle>,
    FrameHistoryHandle,
    FrameHistoryStatus,
) {
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
                context.render_size(),
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
    record.replace_visibility_static_index(context.visibility_context().static_index().clone());

    let previous_available = previous_handle.is_some()
        && allocated_history.is_none()
        && context.history_invalidation_reason().is_none();
    let invalidation_reason = if previous_available {
        None
    } else {
        context.history_invalidation_reason()
    };
    let history_status = FrameHistoryStatus::new(
        Some(history_handle),
        previous_handle,
        previous_available,
        invalidation_reason,
        context.size(),
        context.render_size(),
    );

    (previous_handle, history_handle, history_status)
}
