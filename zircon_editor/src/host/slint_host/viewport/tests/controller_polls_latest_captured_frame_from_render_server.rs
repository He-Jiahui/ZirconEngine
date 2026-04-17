use std::sync::Arc;

use zircon_math::UVec2;

use super::super::SlintViewportController;
use super::fake_render_server::FakeRenderServer;
use super::test_extract::test_extract;

#[test]
fn controller_polls_latest_captured_frame_from_render_server() {
    let server = Arc::new(FakeRenderServer::default());
    let controller = SlintViewportController::new_with_server(server.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(160, 90))
        .unwrap();

    let image = controller.poll_image();

    assert!(image.is_some());
    assert_eq!(server.state.lock().unwrap().capture_requests, 1);
}
