use super::*;

#[test]
fn perspective_pointer_location_builds_center_camera_ray() {
    let camera = test_camera(ProjectionMode::Perspective);
    let viewport = UVec2::new(1280, 720);

    let ray = ray_from_viewport_point(&camera, viewport, Vec2::new(640.0, 360.0))
        .expect("center pointer should produce a camera ray");

    assert_eq!(ray.origin, camera.transform.translation);
    assert!(ray
        .direction
        .abs_diff_eq(camera.transform.forward(), 0.0001));
}

#[test]
fn ray_map_respects_pointer_viewport_and_camera_activity() {
    let pointer = PointerId::new(7);
    let viewport = RenderViewportHandle::new(2);
    let other_viewport = RenderViewportHandle::new(3);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(pointer, viewport, Vec2::new(320.0, 180.0)),
            PointerLocation::new(PointerId::new(8), other_viewport, Vec2::new(320.0, 180.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                viewport,
                UVec2::new(640, 360),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                viewport,
                UVec2::new(640, 360),
                test_camera(ProjectionMode::Perspective),
            )
            .inactive(),
        ],
    );

    assert_eq!(ray_map.len(), 1);
    assert!(ray_map.get(&RayId::new(11, pointer, viewport)).is_some());
    assert!(ray_map.get(&RayId::new(12, pointer, viewport)).is_none());
    assert!(ray_map
        .get(&RayId::new(11, PointerId::new(8), other_viewport))
        .is_none());
}

#[test]
fn ray_map_builds_rays_for_two_pointers_and_two_active_cameras() {
    let first_pointer = PointerId::new(1);
    let second_pointer = PointerId::new(2);
    let viewport = RenderViewportHandle::new(1);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(first_pointer, viewport, Vec2::new(25.0, 25.0)),
            PointerLocation::new(second_pointer, viewport, Vec2::new(75.0, 75.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
        ],
    );

    assert_eq!(ray_map.len(), 4);
    assert!(ray_map
        .get(&RayId::new(11, first_pointer, viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(12, first_pointer, viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(11, second_pointer, viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(12, second_pointer, viewport))
        .is_some());
}

#[test]
fn ray_map_keeps_same_pointer_locations_scoped_by_viewport() {
    let pointer = PointerId::new(1);
    let first_viewport = RenderViewportHandle::new(1);
    let second_viewport = RenderViewportHandle::new(2);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(pointer, first_viewport, Vec2::new(25.0, 25.0)),
            PointerLocation::new(pointer, second_viewport, Vec2::new(75.0, 75.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                first_viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                second_viewport,
                UVec2::new(100, 100),
                test_camera(ProjectionMode::Perspective),
            ),
        ],
    );

    assert_eq!(ray_map.len(), 2);
    assert!(ray_map
        .get(&RayId::new(11, pointer, first_viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(12, pointer, second_viewport))
        .is_some());
    assert!(ray_map
        .get(&RayId::new(11, pointer, second_viewport))
        .is_none());
    assert!(ray_map
        .get(&RayId::new(12, pointer, first_viewport))
        .is_none());
}

#[test]
fn ray_generation_uses_actual_viewport_aspect_for_off_center_pointers() {
    let mut camera = test_camera(ProjectionMode::Perspective);
    camera.aspect_ratio = 16.0 / 9.0;
    let square_viewport = UVec2::new(100, 100);

    let ray = ray_from_viewport_point(&camera, square_viewport, Vec2::new(100.0, 50.0))
        .expect("right-edge pointer should produce a camera ray");

    let half_fov_tan = (camera.fov_y_radians * 0.5).tan();
    let expected = Vec3::new(half_fov_tan, 0.0, -1.0).normalize();
    assert!(ray.direction.abs_diff_eq(expected, 0.0001));
}
