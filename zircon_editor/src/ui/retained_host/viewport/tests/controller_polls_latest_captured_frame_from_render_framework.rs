use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use zircon_runtime_interface::math::UVec2;

use super::super::RetainedViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_polls_latest_captured_frame_from_render_framework() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    let image = controller.poll_captured_frame();

    assert!(image.is_some());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 1);
}

#[test]
fn controller_does_not_republish_unchanged_captured_frame() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    assert!(controller.poll_captured_frame().is_some());
    assert!(controller.poll_captured_frame().is_none());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 2);
}

#[test]
fn controller_does_not_republish_cached_image_when_no_new_frame_is_available() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    assert!(controller.poll_captured_frame().is_some());
    framework.state.lock().unwrap().captures.clear();

    assert!(controller.poll_captured_frame().is_none());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 2);
}

#[test]
fn controller_does_not_republish_cached_image_when_capture_fails() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();
    assert!(controller.poll_captured_frame().is_some());
    framework.state.lock().unwrap().capture_error = Some("planned capture failure".to_string());

    assert!(controller.poll_captured_frame().is_none());
    assert!(
        controller
            .take_error()
            .is_some_and(|error| error.contains("planned capture failure"))
    );
    assert_eq!(framework.state.lock().unwrap().capture_requests, 2);
}

#[test]
fn capture_poll_does_not_wait_for_a_viewport_submit_operation() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = Arc::new(RetainedViewportController::new_with_framework(
        framework.clone(),
    ));
    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    let (submit_started, submit_release) = framework.block_next_submit();
    let submit_controller = Arc::clone(&controller);
    let submitted = thread::spawn(move || {
        submit_controller.submit_extract(test_extract(), UVec2::new(160, 90))
    });
    submit_started
        .recv_timeout(Duration::from_secs(5))
        .expect("fixture submit should reach the framework gate");

    let (capture_sender, capture_receiver) = channel();
    let poll_controller = Arc::clone(&controller);
    let polled = thread::spawn(move || {
        capture_sender
            .send(poll_controller.poll_captured_frame())
            .expect("capture poll result should be observable");
    });

    assert!(
        capture_receiver
            .recv_timeout(Duration::from_millis(100))
            .expect("capture poll must not wait for a viewport submit operation")
            .is_some()
    );

    submit_release
        .send(())
        .expect("fixture submit should accept release");
    polled.join().expect("capture poll thread should not panic");
    assert!(
        submitted
            .join()
            .expect("submit thread should not panic")
            .is_ok()
    );
}
