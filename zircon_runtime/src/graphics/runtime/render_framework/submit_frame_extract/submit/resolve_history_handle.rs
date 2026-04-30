use crate::core::framework::render::{FrameHistoryHandle, RenderViewportHandle};

use crate::graphics::runtime::render_framework::render_framework_state::RenderFrameworkState;

use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) struct ResolvedHistoryHandle {
    allocated_history: Option<FrameHistoryHandle>,
    current_history_handle: Option<FrameHistoryHandle>,
}

impl ResolvedHistoryHandle {
    fn new(
        allocated_history: Option<FrameHistoryHandle>,
        current_history_handle: Option<FrameHistoryHandle>,
    ) -> Self {
        Self {
            allocated_history,
            current_history_handle,
        }
    }

    pub(super) fn allocated_history(&self) -> Option<FrameHistoryHandle> {
        self.allocated_history
    }

    pub(super) fn current_history_handle(&self) -> Option<FrameHistoryHandle> {
        self.current_history_handle
    }
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

    ResolvedHistoryHandle::new(allocated_history, current_history_handle)
}

fn should_rotate_history(
    state: &RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> bool {
    state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history())
        .is_none_or(|history| {
            !history.is_compatible(
                context.size(),
                context.pipeline_handle(),
                &context.compiled_pipeline().history_bindings,
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
        .and_then(|record| record.history().map(|history| history.handle()))
}
