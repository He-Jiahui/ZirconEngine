use super::*;

#[test]
fn dynamic_runtime_camera_controller_scroll_uses_runtime_orbit_controller() {
    let mut scene = Scene::new();
    let camera = scene.active_camera();
    let before = scene.find_node(camera).unwrap().transform;
    let mut controller = RuntimeCameraController::new(UVec2::new(800, 600));

    controller.set_orbit_target(Vec3::ZERO);
    controller.scrolled(&mut scene, 1.0);

    let after = scene.find_node(camera).unwrap().transform;
    assert!(after.translation.length() < before.translation.length());
}

#[test]
fn dynamic_runtime_camera_controller_reads_only_the_camera_transform() {
    let source = include_str!("../camera_controller.rs");
    let full_node_read = ["scene.find_node(", "scene.active_camera())"].concat();
    assert!(
        !source.contains(&full_node_read),
        "camera input must not project and clone a full SceneNode"
    );
    let local_transform_read = ["scene.", "local_transform(camera)"].concat();
    assert_eq!(source.matches(&local_transform_read).count(), 3);
}

#[test]
fn editor_camera_override_changes_only_the_render_extract_view() {
    let scene = Scene::new();
    let active_camera = scene.active_camera();
    let world_transform_before = scene.world_transform(active_camera).unwrap();
    let mut extract = scene.to_render_frame_extract();
    let original_pipeline = extract.view.camera.core_pipeline;
    let original_exposure = extract.view.camera.exposure_ev100;
    let mut controller = RuntimeCameraController::new(UVec2::new(800, 600));
    let camera = ZrRuntimeViewportCameraV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::core::math::Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)),
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
        60.0_f32.to_radians(),
        12.0,
        0.25,
        500.0,
    );

    assert!(controller.apply_editor_camera(camera).unwrap());
    controller.apply_editor_camera_to_extract(&mut extract);

    assert_eq!(extract.view.camera.transform, camera.transform);
    assert_eq!(
        extract.view.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(extract.view.camera.core_pipeline, original_pipeline);
    assert_eq!(extract.view.camera.exposure_ev100, original_exposure);
    assert_eq!(
        scene.world_transform(active_camera),
        Some(world_transform_before)
    );
}
