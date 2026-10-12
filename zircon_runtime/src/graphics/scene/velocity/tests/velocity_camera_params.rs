use crate::core::framework::render::{
    ProjectionMode, TemporalJitterSample, ViewportCameraSnapshot,
};
use crate::core::math::{Mat4, Quat, Transform, UVec2, Vec2, Vec3};

use super::VelocityCameraParams;

#[test]
fn render_velocity_camera_params_enable_with_compatible_previous_camera() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        transform: Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(params.is_enabled());
    assert_eq!(params.viewport_and_flags, [1280, 720, 1, 1]);
    assert_ne!(
        params.current_clip_from_world,
        params.previous_clip_from_world
    );
}

#[test]
fn render_velocity_camera_params_enable_during_continuous_fov_change() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        fov_y_radians: 65.0_f32.to_radians(),
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(params.is_enabled());
}

#[test]
fn render_velocity_camera_params_disable_on_projection_or_feature_mismatch() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Orthographic,
        ..ViewportCameraSnapshot::default()
    };

    let incompatible =
        VelocityCameraParams::from_cameras(UVec2::new(64, 64), &current, &previous, true);
    let disabled =
        VelocityCameraParams::from_cameras(UVec2::new(64, 64), &current, &current, false);

    assert!(!incompatible.is_enabled());
    assert!(!disabled.is_enabled());
}

#[test]
fn render_velocity_camera_params_use_unjittered_camera_matrices() {
    let jittered = ViewportCameraSnapshot {
        temporal_jitter: TemporalJitterSample {
            offset_pixels: Vec2::new(0.5, -0.25),
            sequence_index: 3,
        },
        ..ViewportCameraSnapshot::default()
    };
    let unjittered = ViewportCameraSnapshot::default();

    let jittered_params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &jittered, &unjittered, true);
    let unjittered_params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &unjittered, &unjittered, true);

    assert_eq!(
        jittered_params.current_clip_from_world,
        unjittered_params.current_clip_from_world
    );
    assert_eq!(
        jittered_params.current_world_from_clip,
        unjittered_params.current_world_from_clip
    );
}

#[test]
fn render_velocity_camera_params_disable_on_fov_cut() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        fov_y_radians: 100.0_f32.to_radians(),
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(
        !params.is_enabled(),
        "large FOV cuts should clear camera motion vectors instead of reprojecting history"
    );
}

#[test]
fn render_velocity_camera_params_disable_on_orthographic_size_cut() {
    let current = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Orthographic,
        ..ViewportCameraSnapshot::default()
    };
    let previous = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Orthographic,
        ortho_size: 12.0,
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(
        !params.is_enabled(),
        "large orthographic size cuts should clear camera motion vectors"
    );
}

#[test]
fn render_velocity_camera_params_disable_on_clip_plane_cut() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        z_near: 1.0,
        z_far: 20.0,
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(
        !params.is_enabled(),
        "large clip-plane cuts should clear camera motion vectors"
    );
}

#[test]
fn render_velocity_camera_params_disable_on_invalid_projection_without_nan_matrices() {
    let current = ViewportCameraSnapshot {
        fov_y_radians: f32::NAN,
        ..ViewportCameraSnapshot::default()
    };
    let previous = ViewportCameraSnapshot::default();

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(!params.is_enabled());
    assert_eq!(
        params.current_clip_from_world,
        Mat4::IDENTITY.to_cols_array_2d()
    );
    assert_eq!(
        params.current_world_from_clip,
        Mat4::IDENTITY.to_cols_array_2d()
    );
    assert_eq!(
        params.previous_clip_from_world,
        Mat4::IDENTITY.to_cols_array_2d()
    );
}

#[test]
fn render_velocity_camera_params_disable_on_invalid_clip_range_without_nan_matrices() {
    let current = ViewportCameraSnapshot {
        z_near: 10.0,
        z_far: 1.0,
        ..ViewportCameraSnapshot::default()
    };
    let previous = ViewportCameraSnapshot::default();

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(!params.is_enabled());
    assert_eq!(
        params.current_clip_from_world,
        Mat4::IDENTITY.to_cols_array_2d()
    );
    assert_eq!(
        params.current_world_from_clip,
        Mat4::IDENTITY.to_cols_array_2d()
    );
    assert_eq!(
        params.previous_clip_from_world,
        Mat4::IDENTITY.to_cols_array_2d()
    );
}

#[test]
fn render_velocity_camera_params_disable_on_camera_cut_translation() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(
        !params.is_enabled(),
        "large camera cuts should clear motion vectors instead of reprojecting stale history"
    );
}

#[test]
fn render_velocity_camera_params_disable_on_camera_cut_rotation() {
    let current = ViewportCameraSnapshot::default();
    let previous = ViewportCameraSnapshot {
        transform: Transform::identity()
            .with_rotation(Quat::from_rotation_y(120.0_f32.to_radians())),
        ..ViewportCameraSnapshot::default()
    };

    let params =
        VelocityCameraParams::from_cameras(UVec2::new(1280, 720), &current, &previous, true);

    assert!(
        !params.is_enabled(),
        "large camera rotations should clear motion vectors instead of blurring a camera cut"
    );
}
