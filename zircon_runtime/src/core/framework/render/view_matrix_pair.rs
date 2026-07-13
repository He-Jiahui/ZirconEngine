use crate::core::math::{view_matrix, Mat4, Real, UVec2, Vec3};

use super::{aspect_ratio_from_viewport_size, ProjectionMode, ViewportCameraSnapshot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewProjectionMatrixPair {
    pub clip_from_world_jittered: Mat4,
    pub clip_from_world_unjittered: Mat4,
}

impl ViewProjectionMatrixPair {
    pub fn from_camera(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Self {
        let projection = projection_from_camera(camera, viewport_size);
        let clip_from_world_unjittered = projection * view_matrix(camera.transform);
        let jitter = camera.temporal_jitter.offset_pixels;
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
}

fn projection_from_camera(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Mat4 {
    if let Some(projection) = camera.projection_override {
        return projection;
    }
    let aspect = aspect_ratio_from_viewport_size(viewport_size);
    match camera.projection_mode {
        ProjectionMode::Perspective => Mat4::perspective_rh(
            camera.fov_y_radians,
            aspect.max(0.001),
            camera.z_near.max(0.001),
            camera.z_far,
        ),
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        TemporalJitterSample, ViewProjectionMatrixPair, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec2, Vec3};

    #[test]
    fn render_taa_matrix_pair_is_identical_without_jitter() {
        let camera = ViewportCameraSnapshot::default();

        let pair = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(1280, 720));

        assert_eq!(
            pair.clip_from_world_jittered,
            pair.clip_from_world_unjittered
        );
    }

    #[test]
    fn render_taa_matrix_pair_applies_pixel_jitter_in_clip_space() {
        let camera = ViewportCameraSnapshot {
            temporal_jitter: TemporalJitterSample {
                offset_pixels: Vec2::new(0.5, -0.25),
                sequence_index: 3,
            },
            ..ViewportCameraSnapshot::default()
        };

        let pair = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(100, 50));
        let world_point = Vec3::new(0.0, 0.0, -1.0);
        let unjittered_origin = pair.clip_from_world_unjittered.project_point3(world_point);
        let jittered_origin = pair.clip_from_world_jittered.project_point3(world_point);

        assert_close(jittered_origin.x - unjittered_origin.x, 0.01);
        assert_close(jittered_origin.y - unjittered_origin.y, -0.01);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
