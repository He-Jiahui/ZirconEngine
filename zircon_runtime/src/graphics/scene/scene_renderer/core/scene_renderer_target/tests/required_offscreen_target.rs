use super::{require_offscreen_target, require_offscreen_target_mut};
use crate::graphics::types::GraphicsError;

#[test]
fn missing_offscreen_target_returns_a_typed_graphics_error() {
    let error = require_offscreen_target(None)
        .err()
        .expect("a frame entry without its installed target must fail closed");

    assert!(matches!(error, GraphicsError::OffscreenTargetUnavailable));
}

#[test]
fn missing_mutable_offscreen_target_returns_a_typed_graphics_error() {
    let error = require_offscreen_target_mut(None)
        .err()
        .expect("a mutable frame target lookup must fail closed");

    assert!(matches!(error, GraphicsError::OffscreenTargetUnavailable));
}
