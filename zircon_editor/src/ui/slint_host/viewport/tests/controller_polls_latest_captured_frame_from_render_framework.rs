use std::sync::Arc;

use zircon_runtime::core::math::UVec2;

use super::super::SlintViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_polls_latest_captured_frame_from_render_framework() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    let image = controller.poll_image();

    assert!(image.is_some());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 1);
}

#[test]
fn controller_does_not_republish_unchanged_captured_frame() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    assert!(controller.poll_image().is_some());
    assert!(controller.poll_image().is_none());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 2);
}

#[test]
fn controller_does_not_republish_cached_image_when_no_new_frame_is_available() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    assert!(controller.poll_image().is_some());
    framework.state.lock().unwrap().captures.clear();

    assert!(controller.poll_image().is_none());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 2);
}
