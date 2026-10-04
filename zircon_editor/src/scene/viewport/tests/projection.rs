use super::*;

#[test]
fn shared_projection_context_projects_center_and_reuses_camera_scale() {
    let camera = ViewportCameraSnapshot::default();
    let context = ViewportProjectionContext::new(&camera, UVec2::new(800, 600));

    let center = context
        .projected_point(Vec3::new(0.0, 0.0, -5.0))
        .expect("point in front of the camera should project");

    assert!((center.position.x - 400.0).abs() < 0.01);
    assert!((center.position.y - 300.0).abs() < 0.01);
    assert!(context.world_units_per_pixel(Vec3::new(0.0, 0.0, -5.0)) > 0.0);
}

#[test]
fn shared_projection_context_builds_a_center_cursor_ray_from_the_camera() {
    let camera = ViewportCameraSnapshot::default();
    let context = ViewportProjectionContext::new(&camera, UVec2::new(800, 600));
    let ray = context.spatial_ray_at(Vec2::new(400.0, 300.0));

    assert_eq!(ray.origin, camera.transform.translation);
    assert!(ray.direction.dot(camera.transform.forward()) > 0.999);
    assert!(ray.max_distance >= camera.z_near);
}

#[test]
fn shared_projection_context_fails_closed_for_an_invalid_perspective_ray() {
    let mut camera = ViewportCameraSnapshot::default();
    camera.fov_y_radians = f32::NAN;
    let ray = ViewportProjectionContext::new(&camera, UVec2::new(800, 600))
        .spatial_ray_at(Vec2::new(400.0, 300.0));

    assert_eq!(ray.direction, Vec3::ZERO);
}

#[test]
fn shared_projection_context_matches_its_camera_and_normalized_viewport() {
    let camera = ViewportCameraSnapshot::default();
    let context = ViewportProjectionContext::new(&camera, UVec2::new(800, 0));

    assert!(context.matches_camera_and_viewport(&camera, UVec2::new(800, 0)));
    assert!(context.matches_camera_and_viewport(&camera, UVec2::new(800, 1)));

    let mut changed_camera = camera.clone();
    changed_camera.fov_y_radians += 0.1;
    assert!(!context.matches_camera_and_viewport(&changed_camera, UVec2::new(800, 1)));
    assert!(!context.matches_camera_and_viewport(&camera, UVec2::new(801, 1)));
}

#[test]
fn pointer_candidate_pipeline_constructs_projection_context_once() {
    let root = include_str!("../pointer/candidates/precision_candidates_from_layout.rs");
    let leaf_sources = [
        include_str!("../pointer/candidates/handle_candidate.rs"),
        include_str!("../pointer/candidates/projected_ring_segments.rs"),
        include_str!("../pointer/candidates/renderable_candidate.rs"),
        include_str!("../pointer/candidates/scene_gizmo_candidate.rs"),
    ];

    assert_eq!(root.matches("ViewportProjectionContext::new").count(), 1);
    assert!(leaf_sources
        .iter()
        .all(|source| !source.contains("ViewportProjectionContext::new")));
}
