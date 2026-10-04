//! 解析历史时保留旧句柄直到新提交完成；不兼容时分配新句柄并传递失效原因。
use crate::core::framework::render::{
    FrameHistoryHandle, FrameHistoryInvalidationReason, RenderViewportHandle,
};

use crate::graphics::runtime::render_framework::render_framework_state::RenderFrameworkState;

use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) struct ResolvedHistoryHandle {
    allocated_history: Option<FrameHistoryHandle>,
    current_history_handle: Option<FrameHistoryHandle>,
    previous_history_available: bool,
}

impl ResolvedHistoryHandle {
    fn new(
        allocated_history: Option<FrameHistoryHandle>,
        current_history_handle: Option<FrameHistoryHandle>,
        previous_history_available: bool,
    ) -> Self {
        Self {
            allocated_history,
            current_history_handle,
            previous_history_available,
        }
    }

    pub(super) fn allocated_history(&self) -> Option<FrameHistoryHandle> {
        self.allocated_history
    }

    pub(super) fn current_history_handle(&self) -> Option<FrameHistoryHandle> {
        self.current_history_handle
    }

    pub(super) fn previous_history_available(&self) -> bool {
        self.previous_history_available
    }
}

pub(super) fn resolve_history_handle(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> ResolvedHistoryHandle {
    let allocated_history =
        history_invalidation_requires_reallocation(context.history_invalidation_reason())
            .then(|| allocate_history_handle(state));
    let current_history_handle =
        allocated_history.or_else(|| current_history_handle(state, viewport, context));
    let previous_history_available = current_history_handle.is_some()
        && allocated_history.is_none()
        && context.history_invalidation_reason().is_none();

    ResolvedHistoryHandle::new(
        allocated_history,
        current_history_handle,
        previous_history_available,
    )
}

const fn history_invalidation_requires_reallocation(
    reason: Option<FrameHistoryInvalidationReason>,
) -> bool {
    !matches!(
        reason,
        None | Some(FrameHistoryInvalidationReason::FrameInputsChanged)
            | Some(FrameHistoryInvalidationReason::CameraCut)
    )
}

fn allocate_history_handle(state: &mut RenderFrameworkState) -> FrameHistoryHandle {
    let handle = FrameHistoryHandle::new(state.next_history_id);
    state.next_history_id += 1;
    handle
}

fn current_history_handle(
    state: &RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Option<FrameHistoryHandle> {
    state.viewports.get(&viewport).and_then(|record| {
        record
            .history(context.camera_history_key())
            .map(|history| history.handle())
    })
}

#[cfg(test)]
#[path = "tests/resolve_history_handle.rs"]
mod tests;
