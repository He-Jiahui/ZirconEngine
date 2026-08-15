use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use zircon_runtime_interface::math::UVec2;

use super::super::RetainedViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_operation_gate_keeps_viewport_alive_without_holding_shared_state() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = Arc::new(RetainedViewportController::new_with_framework(
        framework.clone(),
    ));
    let (submit_started, submit_release) = framework.block_next_submit();
    let submit_controller = Arc::clone(&controller);

    let submitted = thread::spawn(move || {
        submit_controller.submit_extract(test_extract(), UVec2::new(320, 240))
    });
    submit_started
        .recv_timeout(Duration::from_secs(5))
        .expect("fixture submit should reach the framework gate");

    assert!(
        controller.shared.try_lock().is_ok(),
        "framework submission must not retain the controller state mutex"
    );
    assert!(
        controller.viewport_lifecycle.try_lock().is_err(),
        "viewport operation gate must retain the handle until framework submission returns"
    );

    submit_release
        .send(())
        .expect("fixture submit should accept release");
    assert!(
        submitted
            .join()
            .expect("fixture submit thread should not panic")
            .is_ok()
    );
    assert!(
        controller.viewport_lifecycle.try_lock().is_ok(),
        "viewport operation gate must release after framework submission returns"
    );
}

#[test]
fn controller_defers_different_size_recreate_until_the_in_flight_submit_returns() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = Arc::new(RetainedViewportController::new_with_framework(
        framework.clone(),
    ));
    let (submit_started, submit_release) = framework.block_next_submit();
    let first_controller = Arc::clone(&controller);
    let first = thread::spawn(move || {
        first_controller.submit_extract(test_extract(), UVec2::new(320, 240))
    });
    submit_started
        .recv_timeout(Duration::from_secs(5))
        .expect("first submit should reach the framework gate");

    let destroy_started = framework.notify_next_destroy();
    let (second_attempted_sender, second_attempted) = channel();
    let second_controller = Arc::clone(&controller);
    let second = thread::spawn(move || {
        second_attempted_sender
            .send(())
            .expect("second submit attempt should be observable");
        second_controller.submit_extract(test_extract(), UVec2::new(640, 480))
    });
    second_attempted
        .recv_timeout(Duration::from_secs(5))
        .expect("second resize submit should start");

    let destroy_before_release = destroy_started.recv_timeout(Duration::from_millis(100));

    submit_release
        .send(())
        .expect("first submit should accept release");
    let destroy_after_release = matches!(&destroy_before_release, Err(RecvTimeoutError::Timeout))
        .then(|| {
            destroy_started
                .recv_timeout(Duration::from_secs(5))
                .expect("resize should destroy the old viewport after submit returns")
        });
    let first_result = first.join().expect("first submit thread should not panic");
    let second_result = second
        .join()
        .expect("second submit thread should not panic");

    assert!(
        matches!(destroy_before_release, Err(RecvTimeoutError::Timeout)),
        "different-size submission must not destroy the active viewport before submit returns"
    );
    assert_eq!(
        destroy_after_release,
        Some(crate::scene::viewport::RenderViewportHandle::new(1))
    );
    assert!(first_result.is_ok());
    assert!(second_result.is_ok());

    let state = framework.state.lock().unwrap();
    assert_eq!(
        state.destroyed_viewports,
        vec![crate::scene::viewport::RenderViewportHandle::new(1)]
    );
    assert_eq!(
        state.submitted_viewports,
        vec![
            crate::scene::viewport::RenderViewportHandle::new(1),
            crate::scene::viewport::RenderViewportHandle::new(2)
        ]
    );
}
