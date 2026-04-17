use std::sync::Arc;

use zircon_math::UVec2;

use super::super::SlintViewportController;
use super::fake_render_server::FakeRenderServer;
use super::test_extract::test_extract;

#[test]
fn controller_updates_extract_camera_aspect_ratio_to_match_viewport_size() {
    let server = Arc::new(FakeRenderServer::default());
    let controller = SlintViewportController::new_with_server(server.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(300, 150))
        .unwrap();

    assert_eq!(
        server.state.lock().unwrap().submitted_aspect_ratios,
        vec![2.0]
    );
}
