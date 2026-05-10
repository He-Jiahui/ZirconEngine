use crate::core::framework::render::{RenderFrameworkError, RenderViewportHandle};

use super::super::render_framework_state::RenderFrameworkState;
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
