use crate::core::framework::camera_controller::{
    FreeCameraController, FreeCameraInput, FreeCameraSettings, FreeCameraState,
    OrbitCameraController, OrbitCameraInput, PanCameraController, PanCameraInput,
    PanCameraSettings, PanCameraState,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3};

#[test]
fn free_camera_controller_moves_forward_clamps_pitch_and_requests_cursor_grab() {
    let mut controller = FreeCameraController::new(
        FreeCameraSettings {
            sensitivity: 1.0,
            pitch_min: -0.25,
            pitch_max: 0.25,
            ..FreeCameraSettings::default()
        },
        FreeCameraState::default(),
    );

    let output = controller.update(
        Transform::identity(),
        FreeCameraInput {
            delta_seconds: 1.0,
            movement_axis: Vec3::Z,
            look_delta: Vec2::new(0.0, -1000.0),
            look_active: true,
            cursor_grab_active: true,
            cursor_grab_changed: true,
            ..FreeCameraInput::default()
        },
    );

    assert!(output.changed);
    assert!(output.transform.translation.z < -4.9);
    assert!((controller.state().pitch - 0.25).abs() < 0.0001);
    assert!(output.cursor_grab.is_some());
}

#[test]
fn free_camera_controller_scrolls_speed_and_decays_velocity() {
    let mut controller = FreeCameraController::default();
    let transform = Transform::identity();

    let moved = controller.update(
        transform,
        FreeCameraInput {
            delta_seconds: 1.0,
            movement_axis: Vec3::Z,
            scroll_lines: 2.0,
            ..FreeCameraInput::default()
        },
    );
    let speed_after_scroll = controller.state().speed_multiplier;
    let velocity_after_move = controller.state().velocity.length();

    let decayed = controller.update(
        moved.transform,
        FreeCameraInput {
            delta_seconds: 0.01,
            ..FreeCameraInput::default()
        },
    );

    assert!(speed_after_scroll > 1.0);
    assert!(velocity_after_move > 0.0);
    assert!(controller.state().velocity.length() < velocity_after_move);
    assert!(decayed.changed);
}

#[test]
fn pan_camera_controller_translates_rotates_and_clamps_zoom() {
    let mut controller = PanCameraController::new(
        PanCameraSettings {
            min_zoom: 0.5,
            max_zoom: 2.0,
            zoom_speed: 1.0,
            pan_speed: 10.0,
            drag_pan_speed: 100.0,
            ..PanCameraSettings::default()
        },
        PanCameraState {
            zoom_factor: 1.0,
            ..PanCameraState::default()
        },
    );

    let output = controller.update(
        Transform::identity(),
        PanCameraInput {
            delta_seconds: 1.0,
            pan_axis: Vec2::X,
            drag_delta: Vec2::new(16.0, -8.0),
            zoom_delta: 10.0,
            rotate_axis: 1.0,
            viewport_size: UVec2::new(160, 90),
        },
    );

    assert!(output.changed);
    assert!(output.transform.translation.x.abs() > 0.1);
    assert_eq!(controller.state().zoom_factor, 0.5);
    assert_eq!(output.transform.scale, Vec3::splat(0.5));
    assert_ne!(output.transform.rotation, Transform::identity().rotation);
}

#[test]
fn orbit_camera_controller_orbits_pans_and_zooms_around_target() {
    let mut controller = OrbitCameraController::with_target(Vec3::ZERO);
    let initial = Transform::looking_at(Vec3::new(0.0, 0.0, 8.0), Vec3::ZERO, Vec3::Y);

    let orbited = controller.update(
        initial,
        OrbitCameraInput::orbit(Vec2::ZERO, Vec2::new(20.0, 5.0))
            .with_viewport_size(UVec2::new(800, 600)),
    );
    let initial_distance = initial.translation.length();
    let orbit_distance = (orbited.transform.translation - controller.target()).length();

    let panned = controller.update(
        orbited.transform,
        OrbitCameraInput::pan(Vec2::ZERO, Vec2::new(10.0, -4.0))
            .with_viewport_size(UVec2::new(800, 600)),
    );
    let target_after_pan = controller.target();

    let zoomed = controller.update(
        panned.transform,
        OrbitCameraInput::zoom(1.0).with_viewport_size(UVec2::new(800, 600)),
    );
    let distance_after_pan = (panned.transform.translation - target_after_pan).length();
    let distance_after_zoom = (zoomed.transform.translation - target_after_pan).length();

    assert!(orbited.changed);
    assert!((orbit_distance - initial_distance).abs() < 0.0001);
    assert_ne!(target_after_pan, Vec3::ZERO);
    assert!(panned.changed);
    assert!(distance_after_zoom < distance_after_pan);
}
