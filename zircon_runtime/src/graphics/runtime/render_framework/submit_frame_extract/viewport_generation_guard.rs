//! 锁外生成的提交上下文携带视口代际；锁内读取或写回记录前须确认视口未变化。
use std::collections::HashMap;

use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::render_framework_state::RenderFrameworkState;
use super::super::viewport_record::ViewportRecord;
use super::frame_submission_context::FrameSubmissionContext;

// 锁外预检后、首次锁内资源准备前调用，防止重建视口沿用旧上下文。
pub(super) fn validate_viewport_generation(
    state: &RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Result<(), RenderFrameworkError> {
    let record = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?;
    let actual_generation = record.generation();
    if actual_generation != context.viewport_generation() {
        return Err(RenderFrameworkError::ViewportChanged {
            viewport: viewport.raw(),
            expected_generation: context.viewport_generation(),
            actual_generation,
        });
    }
    Ok(())
}

pub(super) fn viewport_record_mut_after_generation_check<'a>(
    state: &'a mut RenderFrameworkState,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Result<&'a mut ViewportRecord, RenderFrameworkError> {
    viewport_record_mut_after_generation_check_in(&mut state.viewports, viewport, context)
}

pub(super) fn viewport_record_mut_after_generation_check_in<'a>(
    viewports: &'a mut HashMap<RenderViewportHandle, ViewportRecord>,
    viewport: RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Result<&'a mut ViewportRecord, RenderFrameworkError> {
    let record = viewports
        .get_mut(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?;
    let actual_generation = record.generation();
    if actual_generation != context.viewport_generation() {
        return Err(RenderFrameworkError::ViewportChanged {
            viewport: viewport.raw(),
            expected_generation: context.viewport_generation(),
            actual_generation,
        });
    }
    Ok(record)
}

#[cfg(test)]
#[path = "tests/viewport_generation_guard.rs"]
mod tests;
