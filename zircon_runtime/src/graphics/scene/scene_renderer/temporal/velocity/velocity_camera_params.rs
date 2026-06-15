use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    ProjectionMode, ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::{Mat4, UVec2};

const VELOCITY_CAMERA_ENABLED: u32 = 1;
const VELOCITY_CAMERA_PERSPECTIVE: u32 = 1;
// Implausible frame-to-frame camera deltas are treated as cuts and clear velocity.
const VELOCITY_CAMERA_MAX_TRANSLATION_FAR_PLANE_FRACTION: f32 = 0.2;
const VELOCITY_CAMERA_MAX_ROTATION_RADIANS: f32 = core::f32::consts::FRAC_PI_3;
const VELOCITY_CAMERA_MAX_FOV_DELTA_RADIANS: f32 = core::f32::consts::PI / 12.0;
const VELOCITY_CAMERA_MIN_FOV_RADIANS: f32 = core::f32::consts::PI / 180.0;
const VELOCITY_CAMERA_MAX_FOV_RADIANS: f32 =
    core::f32::consts::PI - VELOCITY_CAMERA_MIN_FOV_RADIANS;
const VELOCITY_CAMERA_MAX_ORTHO_SIZE_RELATIVE_DELTA: f32 = 0.25;
const VELOCITY_CAMERA_MAX_CLIP_PLANE_RELATIVE_DELTA: f32 = 0.5;
const VELOCITY_CAMERA_MIN_PROJECTION_PARAMETER: f32 = 0.001;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer) struct VelocityCameraParams {
    pub(in crate::graphics::scene::scene_renderer) viewport_and_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer) current_clip_from_world: [[f32; 4]; 4],
    pub(in crate::graphics::scene::scene_renderer) current_world_from_clip: [[f32; 4]; 4],
    pub(in crate::graphics::scene::scene_renderer) previous_clip_from_world: [[f32; 4]; 4],
}

impl VelocityCameraParams {
    pub(in crate::graphics::scene::scene_renderer) fn from_cameras(
        viewport_size: UVec2,
        current: &ViewportCameraSnapshot,
        previous: &ViewportCameraSnapshot,
        enabled: bool,
    ) -> Self {
        let mut enabled = enabled
            && velocity_camera_projection_compatible(current, previous)
            && velocity_camera_projection_shape_compatible(current, previous)
            && velocity_camera_history_compatible(current, previous);
        let (current_clip_from_world, current_world_from_clip, previous_clip_from_world) =
            if enabled {
                let current_clip_from_world = camera_clip_from_world(current, viewport_size);
                let current_world_from_clip = current_clip_from_world.inverse();
                let previous_clip_from_world = camera_clip_from_world(previous, viewport_size);
                if velocity_camera_matrix_finite(current_clip_from_world)
                    && velocity_camera_matrix_finite(current_world_from_clip)
                    && velocity_camera_matrix_finite(previous_clip_from_world)
                {
                    (
                        current_clip_from_world,
                        current_world_from_clip,
                        previous_clip_from_world,
                    )
                } else {
                    enabled = false;
                    (Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY)
                }
            } else {
                (Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY)
            };
        let projection_flag = if matches!(current.projection_mode, ProjectionMode::Perspective) {
            VELOCITY_CAMERA_PERSPECTIVE
        } else {
            0
        };

        Self {
            viewport_and_flags: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                if enabled { VELOCITY_CAMERA_ENABLED } else { 0 },
                projection_flag,
            ],
            current_clip_from_world: current_clip_from_world.to_cols_array_2d(),
            current_world_from_clip: current_world_from_clip.to_cols_array_2d(),
            previous_clip_from_world: previous_clip_from_world.to_cols_array_2d(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn is_enabled(self) -> bool {
        self.viewport_and_flags[2] == VELOCITY_CAMERA_ENABLED
    }

    pub(in crate::graphics::scene::scene_renderer) fn previous_clip_from_world(
        self,
    ) -> [[f32; 4]; 4] {
        self.previous_clip_from_world
    }
}

fn velocity_camera_projection_compatible(
    current: &ViewportCameraSnapshot,
    previous: &ViewportCameraSnapshot,
) -> bool {
    current.projection_mode == previous.projection_mode
        && current.target == previous.target
        && current.viewport == previous.viewport
        && current.dynamic_resolution == previous.dynamic_resolution
}

fn velocity_camera_projection_shape_compatible(
    current: &ViewportCameraSnapshot,
    previous: &ViewportCameraSnapshot,
) -> bool {
    if !velocity_camera_clip_range_valid(current)
        || !velocity_camera_clip_range_valid(previous)
        || !velocity_camera_relative_parameter_compatible(
            current.z_near,
            previous.z_near,
            VELOCITY_CAMERA_MAX_CLIP_PLANE_RELATIVE_DELTA,
        )
        || !velocity_camera_relative_parameter_compatible(
            current.z_far,
            previous.z_far,
            VELOCITY_CAMERA_MAX_CLIP_PLANE_RELATIVE_DELTA,
        )
    {
        return false;
    }

    match current.projection_mode {
        ProjectionMode::Perspective => {
            let fov_delta = (current.fov_y_radians - previous.fov_y_radians).abs();
            velocity_camera_fov_valid(current.fov_y_radians)
                && velocity_camera_fov_valid(previous.fov_y_radians)
                && fov_delta.is_finite()
                && fov_delta <= VELOCITY_CAMERA_MAX_FOV_DELTA_RADIANS
        }
        ProjectionMode::Orthographic => velocity_camera_relative_parameter_compatible(
            current.ortho_size,
            previous.ortho_size,
            VELOCITY_CAMERA_MAX_ORTHO_SIZE_RELATIVE_DELTA,
        ),
    }
}

fn velocity_camera_clip_range_valid(camera: &ViewportCameraSnapshot) -> bool {
    camera.z_near.is_finite()
        && camera.z_far.is_finite()
        && camera.z_near > 0.0
        && camera.z_far > camera.z_near
}

fn velocity_camera_fov_valid(fov_y_radians: f32) -> bool {
    fov_y_radians.is_finite()
        && fov_y_radians >= VELOCITY_CAMERA_MIN_FOV_RADIANS
        && fov_y_radians <= VELOCITY_CAMERA_MAX_FOV_RADIANS
}

fn velocity_camera_relative_parameter_compatible(
    current: f32,
    previous: f32,
    max_delta_fraction: f32,
) -> bool {
    if !current.is_finite() || !previous.is_finite() || current <= 0.0 || previous <= 0.0 {
        return false;
    }

    let baseline = current
        .abs()
        .max(previous.abs())
        .max(VELOCITY_CAMERA_MIN_PROJECTION_PARAMETER);
    ((current - previous).abs() / baseline) <= max_delta_fraction
}

fn velocity_camera_history_compatible(
    current: &ViewportCameraSnapshot,
    previous: &ViewportCameraSnapshot,
) -> bool {
    let current_translation = current.transform.translation;
    let previous_translation = previous.transform.translation;
    let translation_delta = current_translation.distance(previous_translation);
    if !translation_delta.is_finite() {
        return false;
    }

    let far_plane = current
        .z_far
        .min(previous.z_far)
        .max(current.z_near.max(previous.z_near));
    let max_translation_delta =
        (far_plane * VELOCITY_CAMERA_MAX_TRANSLATION_FAR_PLANE_FRACTION).max(0.001);
    if translation_delta > max_translation_delta {
        return false;
    }

    let rotation_delta = current
        .transform
        .rotation
        .angle_between(previous.transform.rotation);
    rotation_delta.is_finite() && rotation_delta <= VELOCITY_CAMERA_MAX_ROTATION_RADIANS
}

fn velocity_camera_matrix_finite(matrix: Mat4) -> bool {
    matrix
        .to_cols_array()
        .into_iter()
        .all(|value| value.is_finite())
}

fn camera_clip_from_world(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Mat4 {
    ViewProjectionMatrixPair::from_camera(camera, viewport_size).clip_from_world_unjittered
}

#[cfg(test)]
mod tests {
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
        let unjittered_params = VelocityCameraParams::from_cameras(
            UVec2::new(1280, 720),
            &unjittered,
            &unjittered,
            true,
        );

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
}
