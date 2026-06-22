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
    validate_viewport_generation(state, viewport, context)?;
    state
        .viewports
        .get_mut(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })
}
