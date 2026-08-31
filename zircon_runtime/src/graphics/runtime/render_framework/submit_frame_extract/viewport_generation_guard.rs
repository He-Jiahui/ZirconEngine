use std::collections::HashMap;

use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::render_framework_state::RenderFrameworkState;
use super::super::viewport_record::ViewportRecord;
use super::frame_submission_context::FrameSubmissionContext;

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
mod tests {
    #[test]
    fn optimization_batch_fj_runtime466_mutable_generation_guard_uses_one_viewport_lookup() {
        let source = include_str!("viewport_generation_guard.rs");
        let mutable_guard = source
            .split("fn viewport_record_mut_after_generation_check_in")
            .nth(1)
            .expect("mutable generation guard source");
        let nested_validation =
            concat!("validate_viewport_generation(", "state, viewport, context");

        assert!(!mutable_guard.contains(nested_validation));
        assert_eq!(mutable_guard.matches(".get_mut(&viewport)").count(), 1);
    }
}
