use zircon_runtime::scene::components::NodeKind;

use super::*;

#[test]
fn accepted_parent_transform_resynchronizes_the_active_camera_world_transform() {
    let mut scene = Scene::new();
    let active_camera = scene.active_camera();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    scene
        .set_parent_checked(active_camera, Some(parent))
        .unwrap();
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.reset_camera_from_scene(Some(&scene));
    let before = controller.current_camera(&scene).transform;
    let active_camera_transform_before = scene.world_transform(active_camera);
    let target_world = Transform::from_translation(Vec3::new(3.0, 0.0, 0.0));

    scene.update_transform(parent, target_world).unwrap();
    controller.resync_after_interactive_transform(
        &scene,
        parent,
        active_camera,
        active_camera_transform_before,
    );

    let expected = scene.world_transform(active_camera).unwrap();
    assert_ne!(expected, before);
    assert_eq!(controller.current_camera(&scene).transform, expected);
    assert_eq!(controller.orbit_target(), target_world.translation);
}

#[test]
fn unrelated_transform_preserves_the_navigated_editor_camera() {
    let mut scene = Scene::new();
    let target = scene.spawn_node(NodeKind::Empty).unwrap();
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.reset_camera_from_scene(Some(&scene));
    let navigated = Transform::from_translation(Vec3::new(11.0, 7.0, 3.0));
    controller.state.camera.as_mut().unwrap().transform = navigated;
    let active_camera = scene.active_camera();
    let active_camera_transform_before = scene.world_transform(active_camera);
    let target_world = Transform::from_translation(Vec3::new(2.0, 4.0, 6.0));

    scene.update_transform(target, target_world).unwrap();
    controller.resync_after_interactive_transform(
        &scene,
        target,
        active_camera,
        active_camera_transform_before,
    );

    assert_eq!(controller.current_camera(&scene).transform, navigated);
    assert_eq!(controller.orbit_target(), target_world.translation);
}
