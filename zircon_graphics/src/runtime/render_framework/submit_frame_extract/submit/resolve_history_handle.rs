use zircon_framework::render::{FrameHistoryHandle, RenderViewportHandle};

use crate::runtime::render_framework::render_framework_state::RenderFrameworkState;

use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) struct ResolvedHistoryHandle {
    pub(super) allocated_history: Option<FrameHistoryHandle>,
    pub(super) current_history_handle: Option<FrameHistoryHandle>,
}

pub(super) fn resolve_history_handle(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> ResolvedHistoryHandle {
    let allocated_history =
        should_rotate_history(state, viewport, context).then(|| allocate_history_handle(state));
    let current_history_handle =
        allocated_history.or_else(|| current_history_handle(state, viewport));

    ResolvedHistoryHandle {
        allocated_history,
        current_history_handle,
    }
}

fn should_rotate_history(
    state: &RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> bool {
    state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history.as_ref())
        .is_none_or(|history| {
            !history.is_compatible(
                context.size,
                context.pipeline_handle,
                &context.compiled_pipeline.history_bindings,
            )
        })
}

fn allocate_history_handle(state: &mut RenderFrameworkState) -> FrameHistoryHandle {
    let handle = FrameHistoryHandle::new(state.next_history_id);
    state.next_history_id += 1;
    handle
}

fn current_history_handle(
    state: &RenderFrameworkState,
    viewport: RenderViewportHandle,
) -> Option<FrameHistoryHandle> {
    state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history.as_ref().map(|history| history.handle))
}
