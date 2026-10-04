use super::{
    normalize_yaw_degrees, OrbitCamera, CAMERA_PITCH_LIMIT_DEGREES, MAX_CAMERA_RADIUS,
    MIN_CAMERA_RADIUS,
};

#[test]
fn initial_angles_preserve_yaw_and_clamp_pitch() {
    let upper = OrbitCamera::from_angles(-120.0, 220.0);
    let lower = OrbitCamera::from_angles(120.0, -220.0);

    assert_eq!(upper.yaw_degrees(), -120.0);
    assert_eq!(upper.pitch_degrees(), CAMERA_PITCH_LIMIT_DEGREES);
    assert_eq!(lower.yaw_degrees(), 120.0);
    assert_eq!(lower.pitch_degrees(), -CAMERA_PITCH_LIMIT_DEGREES);
}

#[test]
fn mouse_wheel_zoom_changes_radius_and_clamps_to_orbit_limits() {
    let mut camera = OrbitCamera::default();
    let initial_radius = camera.radius;

    camera.zoom(1.0);
    assert!(camera.radius < initial_radius);

    for _ in 0..100 {
        camera.zoom(1.0);
    }
    assert_eq!(camera.radius, MIN_CAMERA_RADIUS);

    for _ in 0..100 {
        camera.zoom(-1.0);
    }
    assert_eq!(camera.radius, MAX_CAMERA_RADIUS);
}

#[test]
fn yaw_normalization_preserves_cardinal_orbit_equivalence() {
    assert_eq!(normalize_yaw_degrees(480.0), 120.0);
    assert_eq!(normalize_yaw_degrees(-480.0), -120.0);
    assert_eq!(normalize_yaw_degrees(540.0), 180.0);
    assert_eq!(normalize_yaw_degrees(-540.0), 180.0);

    let mut camera = OrbitCamera::from_angles(0.0, 0.0);
    camera.drag(3_600.0, 0.0);
    assert_eq!(camera.yaw_degrees(), 180.0);
}
