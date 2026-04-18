use zircon_math::{UVec2, Vec2};
use zircon_runtime::scene::{NodeKind, World};

use super::RuntimeCameraController;

#[test]
fn resize_clamps_viewport_size_to_non_zero_extent() {
    let mut controller = RuntimeCameraController::new(UVec2::new(1280, 720));

    controller.resize(UVec2::ZERO);

    assert_eq!(controller.viewport_size(), UVec2::ONE);
}

#[test]
fn right_drag_orbits_active_camera() {
    let (mut scene, selected) = test_scene_with_selected_cube();
    let initial = scene
        .find_node(scene.active_camera())
        .expect("camera")
        .transform;
    let mut controller = RuntimeCameraController::new(UVec2::new(1280, 720));
    controller.set_orbit_target(test_selection_translation(&scene, selected));

    controller.right_pressed(Vec2::new(320.0, 180.0));
    controller.pointer_moved(&mut scene, Vec2::new(360.0, 220.0));
    controller.right_released();

    let updated = scene
        .find_node(scene.active_camera())
        .expect("camera")
        .transform;
    assert_ne!(updated.translation, initial.translation);
}

#[test]
fn middle_drag_pans_camera_and_orbit_target() {
    let (mut scene, selected) = test_scene_with_selected_cube();
    let initial = scene
        .find_node(scene.active_camera())
        .expect("camera")
        .transform;
    let mut controller = RuntimeCameraController::new(UVec2::new(1280, 720));
    controller.set_orbit_target(test_selection_translation(&scene, selected));
    let initial_target = controller.orbit_target();

    controller.middle_pressed(Vec2::new(320.0, 180.0));
    controller.pointer_moved(&mut scene, Vec2::new(360.0, 210.0));
    controller.middle_released();

    let updated = scene
        .find_node(scene.active_camera())
        .expect("camera")
        .transform;
    assert_ne!(updated.translation, initial.translation);
    assert_ne!(controller.orbit_target(), initial_target);
}

#[test]
fn scroll_zoom_moves_camera_toward_orbit_target() {
    let (mut scene, selected) = test_scene_with_selected_cube();
    let mut controller = RuntimeCameraController::new(UVec2::new(1280, 720));
    controller.set_orbit_target(test_selection_translation(&scene, selected));
    let initial_camera = scene
        .find_node(scene.active_camera())
        .expect("camera")
        .transform;
    let initial_distance = (initial_camera.translation - controller.orbit_target()).length();

    controller.scrolled(&mut scene, 1.0);

    let updated_camera = scene
        .find_node(scene.active_camera())
        .expect("camera")
        .transform;
    let updated_distance = (updated_camera.translation - controller.orbit_target()).length();
    assert!(updated_distance < initial_distance);
}

#[test]
fn left_drag_does_not_translate_selected_node() {
    let (mut scene, selected) = test_scene_with_selected_cube();
    let initial = scene.find_node(selected).expect("selected node").transform;
    let mut controller = RuntimeCameraController::new(UVec2::new(1280, 720));
    controller.set_orbit_target(test_selection_translation(&scene, selected));

    controller.left_pressed(Vec2::new(320.0, 180.0));
    controller.pointer_moved(&mut scene, Vec2::new(420.0, 260.0));
    controller.left_released();

    let updated = scene.find_node(selected).expect("selected node").transform;
    assert_eq!(updated.translation, initial.translation);
}

fn test_scene_with_selected_cube() -> (World, u64) {
    let scene = World::new();
    let selected = scene
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Cube))
        .map(|node| node.id)
        .expect("default world should include a cube");
    (scene, selected)
}

fn test_selection_translation(scene: &World, selected: u64) -> zircon_math::Vec3 {
    scene
        .find_node(selected)
        .expect("selected node")
        .transform
        .translation
}
