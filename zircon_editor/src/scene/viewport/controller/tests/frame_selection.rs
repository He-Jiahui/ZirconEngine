use super::*;
use crate::scene::viewport::SceneViewportController;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::UVec2;

#[test]
fn frame_selection_centers_the_active_multiselection_instead_of_the_primary_item() {
    let mut scene = Scene::new();
    let left = scene
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let right = scene
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    scene
        .update_transform(
            left,
            Transform {
                translation: Vec3::new(-8.0, 0.0, 0.0),
                ..Transform::default()
            },
        )
        .unwrap();
    scene
        .update_transform(
            right,
            Transform {
                translation: Vec3::new(8.0, 0.0, 0.0),
                ..Transform::default()
            },
        )
        .unwrap();

    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller
        .selection_mut()
        .replace_active([left, right], Some(left));

    assert!(controller.frame_selection(&scene));
    assert_eq!(controller.orbit_target(), Vec3::ZERO);
    let camera = controller.current_camera(&scene);
    let minimum_distance = 8.0 * FRAME_PERSPECTIVE_PADDING
        / (camera.fov_y_radians * 0.5).sin().abs().max(f32::EPSILON);
    assert!(
        camera.transform.translation.distance(Vec3::ZERO) >= minimum_distance,
        "frame selection must expand the camera enough to contain the multi-selection bounds"
    );
}
