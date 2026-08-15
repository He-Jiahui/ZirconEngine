use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use zircon_runtime::core::framework::render::RenderViewportProduct;
use zircon_runtime_interface::math::UVec2;

use super::super::RetainedViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_polls_each_gpu_viewport_product_once_without_cpu_capture() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();
    framework.state.lock().unwrap().products.insert(
        crate::scene::viewport::RenderViewportHandle::new(1),
        RenderViewportProduct::new(
            crate::scene::viewport::RenderViewportHandle::new(1),
            160,
            90,
            9,
        ),
    );

    let product = controller
        .poll_viewport_product()
        .expect("the produced GPU texture should be published");

    assert_eq!(product.resource_key(), "viewport:1:9");
    assert_eq!(product.width(), 160);
    assert_eq!(product.height(), 90);
    assert!(controller.poll_viewport_product().is_none());
    assert_eq!(framework.state.lock().unwrap().capture_requests, 0);
}

#[test]
fn product_poll_does_not_wait_for_a_viewport_submit_operation() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = Arc::new(RetainedViewportController::new_with_framework(
        framework.clone(),
    ));
    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();
    framework.state.lock().unwrap().products.insert(
        crate::scene::viewport::RenderViewportHandle::new(1),
        RenderViewportProduct::new(
            crate::scene::viewport::RenderViewportHandle::new(1),
            160,
            90,
            10,
        ),
    );

    let (submit_started, submit_release) = framework.block_next_submit();
    let submit_controller = Arc::clone(&controller);
    let submitted = thread::spawn(move || {
        submit_controller.submit_extract(test_extract(), UVec2::new(160, 90))
    });
    submit_started
        .recv_timeout(Duration::from_secs(5))
        .expect("fixture submit should reach the framework gate");

    let (product_sender, product_receiver) = channel();
    let poll_controller = Arc::clone(&controller);
    let polled = thread::spawn(move || {
        product_sender
            .send(poll_controller.poll_viewport_product())
            .expect("product poll result should be observable");
    });

    let product = product_receiver
        .recv_timeout(Duration::from_millis(100))
        .expect("GPU product poll must not wait for a viewport submit operation");
    assert_eq!(product.map(|product| product.generation()), Some(10));

    submit_release
        .send(())
        .expect("fixture submit should accept release");
    polled.join().expect("product poll thread should not panic");
    assert!(
        submitted
            .join()
            .expect("submit thread should not panic")
            .is_ok()
    );
}
