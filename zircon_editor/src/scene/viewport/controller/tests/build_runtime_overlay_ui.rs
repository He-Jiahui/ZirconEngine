use std::sync::Arc;

use zircon_runtime_interface::math::UVec2;

use super::SceneViewportController;

#[test]
fn stable_viewport_hud_generation_reuses_the_same_allocation() {
    let controller = SceneViewportController::new(UVec2::new(1280, 720));

    let first = controller.build_runtime_overlay_ui().unwrap();
    let second = controller.build_runtime_overlay_ui().unwrap();

    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn viewport_hud_key_change_publishes_one_new_allocation() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    let first = controller.build_runtime_overlay_ui().unwrap();

    controller.apply_viewport_size(UVec2::new(960, 540));
    let resized = controller.build_runtime_overlay_ui().unwrap();
    let stable = controller.build_runtime_overlay_ui().unwrap();

    assert!(!Arc::ptr_eq(&first, &resized));
    assert!(Arc::ptr_eq(&resized, &stable));
}
