use std::sync::Arc;

use zircon_runtime::core::math::UVec2;

use super::super::SlintViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_updates_extract_camera_aspect_ratio_to_match_viewport_size() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract(test_extract(), UVec2::new(300, 150))
        .unwrap();

    assert_eq!(
        framework.state.lock().unwrap().submitted_aspect_ratios,
        vec![2.0]
    );
}
