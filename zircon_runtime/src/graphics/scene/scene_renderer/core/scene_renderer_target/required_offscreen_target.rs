use crate::graphics::backend::OffscreenTarget;
use crate::graphics::types::GraphicsError;

pub(in crate::graphics::scene::scene_renderer::core) fn require_offscreen_target(
    target: Option<&OffscreenTarget>,
) -> Result<&OffscreenTarget, GraphicsError> {
    target.ok_or(GraphicsError::OffscreenTargetUnavailable)
}

pub(in crate::graphics::scene::scene_renderer::core) fn require_offscreen_target_mut(
    target: Option<&mut OffscreenTarget>,
) -> Result<&mut OffscreenTarget, GraphicsError> {
    target.ok_or(GraphicsError::OffscreenTargetUnavailable)
}

#[cfg(test)]
#[path = "tests/required_offscreen_target.rs"]
mod tests;
