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
mod tests {
    use super::{require_offscreen_target, require_offscreen_target_mut};
    use crate::graphics::types::GraphicsError;

    #[test]
    fn missing_offscreen_target_returns_a_typed_graphics_error() {
        let error = require_offscreen_target(None)
            .expect_err("a frame entry without its installed target must fail closed");

        assert!(matches!(error, GraphicsError::OffscreenTargetUnavailable));
    }

    #[test]
    fn missing_mutable_offscreen_target_returns_a_typed_graphics_error() {
        let error = require_offscreen_target_mut(None)
            .expect_err("a mutable frame target lookup must fail closed");

        assert!(matches!(error, GraphicsError::OffscreenTargetUnavailable));
    }
}
