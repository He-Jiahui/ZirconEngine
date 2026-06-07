use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    ProjectionMode, RenderDepthOfFieldSettings, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct DepthOfFieldPrepareParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) depth: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) lens: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) coc_output: [f32; 4],
}

impl DepthOfFieldPrepareParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) fn from_camera(
        viewport_size: UVec2,
        camera: &ViewportCameraSnapshot,
        settings: RenderDepthOfFieldSettings,
    ) -> Self {
        let near = camera.z_near.max(0.001);
        let far = camera.z_far.max(near + 0.001);
        let max_radius = settings.max_blur_radius.max(0.0);

        Self {
            viewport: [viewport_size.x.max(1), viewport_size.y.max(1), 0, 0],
            depth: [
                near,
                far,
                1.0 / (far - near).max(0.001),
                if matches!(camera.projection_mode, ProjectionMode::Perspective) {
                    1.0
                } else {
                    0.0
                },
            ],
            lens: [
                settings.focus_distance.max(0.0),
                settings.render_focus_range(),
                settings.aperture.max(0.0),
                settings.render_focal_length_mm(),
            ],
            coc_output: [
                max_radius,
                if max_radius > f32::EPSILON {
                    1.0 / max_radius
                } else {
                    0.0
                },
                0.0,
                1.0,
            ],
        }
    }
}

pub(in crate::graphics::scene::scene_renderer::post_process) fn depth_of_field_prepare_enabled(
    settings: RenderDepthOfFieldSettings,
) -> bool {
    settings.aperture > f32::EPSILON && settings.max_blur_radius > f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_of_field_prepare_params_sanitize_camera_and_lens_values() {
        let camera = ViewportCameraSnapshot {
            z_near: 0.25,
            z_far: 64.0,
            projection_mode: ProjectionMode::Perspective,
            ..Default::default()
        };
        let params = DepthOfFieldPrepareParams::from_camera(
            UVec2::new(1920, 1080),
            &camera,
            RenderDepthOfFieldSettings {
                focus_distance: 8.0,
                focus_range: -1.0,
                aperture: 0.75,
                focal_length_mm: 400.0,
                max_blur_radius: 6.0,
                ..Default::default()
            },
        );

        assert_eq!(params.viewport, [1920, 1080, 0, 0]);
        assert_near(params.depth[0], 0.25);
        assert_near(params.depth[1], 64.0);
        assert_near(params.depth[2], 1.0 / 63.75);
        assert_near(params.depth[3], 1.0);
        assert_near(params.lens[0], 8.0);
        assert_near(params.lens[1], 0.001);
        assert_near(params.lens[2], 0.75);
        assert_near(params.lens[3], 300.0);
        assert_near(params.coc_output[0], 6.0);
        assert_near(params.coc_output[1], 1.0 / 6.0);
        assert_near(params.coc_output[3], 1.0);
    }

    #[test]
    fn depth_of_field_prepare_requires_aperture_and_radius() {
        assert!(!depth_of_field_prepare_enabled(
            RenderDepthOfFieldSettings::default()
        ));
        assert!(!depth_of_field_prepare_enabled(
            RenderDepthOfFieldSettings {
                aperture: 0.5,
                max_blur_radius: 0.0,
                ..Default::default()
            }
        ));
        assert!(!depth_of_field_prepare_enabled(
            RenderDepthOfFieldSettings {
                aperture: 0.0,
                max_blur_radius: 5.0,
                ..Default::default()
            }
        ));
        assert!(depth_of_field_prepare_enabled(RenderDepthOfFieldSettings {
            aperture: 0.5,
            max_blur_radius: 5.0,
            ..Default::default()
        }));
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be near {expected}"
        );
    }
}
