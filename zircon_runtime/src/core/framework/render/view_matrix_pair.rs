use crate::core::math::{view_matrix, Mat4, Real, UVec2, Vec2, Vec3};

use super::{aspect_ratio_from_viewport_size, ProjectionMode, ViewportCameraSnapshot};

/// 同一相机的像素抖动与稳定投影；光栅化用前者，运动历史和屏幕重建用后者。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewProjectionMatrixPair {
    pub clip_from_world_jittered: Mat4,
    pub clip_from_world_unjittered: Mat4,
}

impl ViewProjectionMatrixPair {
    pub fn from_camera(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Self {
        let projection = Self::projection_from_camera(camera, viewport_size);
        let clip_from_world_unjittered = projection * view_matrix(camera.transform);
        let jitter = camera.temporal_jitter.offset_pixels;
        if jitter == Vec2::ZERO {
            return Self {
                clip_from_world_jittered: clip_from_world_unjittered,
                clip_from_world_unjittered,
            };
        }
        let viewport_width = viewport_size.x.max(1) as Real;
        let viewport_height = viewport_size.y.max(1) as Real;
        let jitter_translation = Mat4::from_translation(Vec3::new(
            2.0 * jitter.x / viewport_width,
            2.0 * jitter.y / viewport_height,
            0.0,
        ));
        Self {
            clip_from_world_jittered: jitter_translation * clip_from_world_unjittered,
            clip_from_world_unjittered,
        }
    }

    pub(crate) fn projection_from_camera(
        camera: &ViewportCameraSnapshot,
        viewport_size: UVec2,
    ) -> Mat4 {
        projection_from_camera(camera, viewport_size)
    }
}

fn projection_from_camera(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Mat4 {
    if let Some(projection) = camera.projection_override {
        return projection;
    }
    let aspect = aspect_ratio_from_viewport_size(viewport_size);
    let z_near = camera.z_near.max(0.001);
    let z_far = camera.z_far.max(z_near + 0.001);
    match camera.projection_mode {
        ProjectionMode::Perspective => {
            Mat4::perspective_rh(camera.fov_y_radians, aspect.max(0.001), z_near, z_far)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                z_near,
                z_far,
            )
        }
    }
}

#[cfg(test)]
#[path = "tests/view_matrix_pair.rs"]
mod tests;
