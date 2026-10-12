use crate::core::framework::render::RenderViewportHandle;

use super::GraphicsDebuggerState;

#[test]
fn capture_frame_count_queues_consecutive_frames_for_one_viewport() {
    let viewport = RenderViewportHandle::new(7);
    let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 2);

    assert!(state.request_capture_for_created_viewport_if_needed(viewport));
    assert!(state.should_capture(viewport));
    state.begin_capture();
    state.finish_capture(Some(10), None);

    assert!(state.should_capture(viewport));
    assert!(state.status().capture_pending);
    state.begin_capture();
    state.finish_capture(Some(11), None);

    let status = state.status();
    assert!(!status.capture_pending);
    assert!(!status.active_capture);
    assert_eq!(status.last_capture_frame, Some(11));
}

#[test]
fn zero_capture_frame_count_does_not_arm_the_created_viewport() {
    let viewport = RenderViewportHandle::new(8);
    let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 0);

    assert!(!state.request_capture_for_created_viewport_if_needed(viewport));
    assert!(!state.should_capture(viewport));
}

#[test]
fn failed_capture_cancels_the_remaining_sequence() {
    let viewport = RenderViewportHandle::new(9);
    let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 3);
    state.request_capture_for_created_viewport_if_needed(viewport);

    state.begin_capture();
    state.fail_pending_capture(viewport, "capture failed".to_owned());

    assert!(!state.should_capture(viewport));
    assert!(!state.status().capture_pending);
    assert_eq!(state.status().last_error.as_deref(), Some("capture failed"));
}

#[test]
fn capture_finish_error_does_not_schedule_the_next_sequence_frame() {
    let viewport = RenderViewportHandle::new(10);
    let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 2);
    state.request_capture_for_created_viewport_if_needed(viewport);

    state.begin_capture();
    state.finish_capture(Some(20), Some("stop failed".to_owned()));

    let status = state.status();
    assert!(!status.capture_pending);
    assert!(!status.active_capture);
    assert_eq!(status.last_capture_frame, Some(20));
    assert_eq!(status.last_error.as_deref(), Some("stop failed"));
}
