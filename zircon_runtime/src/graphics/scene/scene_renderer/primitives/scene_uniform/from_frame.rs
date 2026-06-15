use crate::core::framework::render::ViewProjectionMatrixPair;
use crate::core::math::{Mat4, RenderMat4, RenderVec3};

use crate::graphics::scene::scene_renderer::temporal::velocity::velocity_camera_params::VelocityCameraParams;
use crate::graphics::types::ViewportRenderFrame;

use super::super::fallback::{render_mat4_or, render_vec3_or};
use super::SceneUniform;

impl SceneUniform {
    pub(crate) fn from_frame(frame: &ViewportRenderFrame) -> Self {
        let camera = frame.camera();
        let ambient_color = if frame.preview().lighting_enabled {
            authored_ambient_color(frame, RenderVec3::splat(0.2))
        } else {
            RenderVec3::splat(0.55).extend(1.0).to_array()
        };

        let matrix_pair = ViewProjectionMatrixPair::from_camera(
            camera,
            frame.extract.view.effective_render_size(),
        );
        let view_proj = matrix_pair.clip_from_world_jittered;
        let view_proj_unjittered = matrix_pair.clip_from_world_unjittered;
        let (previous_view_proj_unjittered, motion_params) =
            previous_motion_view_projection(frame, camera, view_proj_unjittered);

        Self {
            view_proj: render_mat4_or(view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            view_proj_unjittered: render_mat4_or(view_proj_unjittered, RenderMat4::IDENTITY)
                .to_cols_array_2d(),
            inverse_view_proj: render_mat4_or(view_proj_unjittered.inverse(), RenderMat4::IDENTITY)
                .to_cols_array_2d(),
            ambient_color,
            previous_view_proj_unjittered,
            motion_params,
            jitter_params: jitter_params(camera),
        }
    }
}

fn previous_motion_view_projection(
    frame: &ViewportRenderFrame,
    camera: &crate::core::framework::render::ViewportCameraSnapshot,
    fallback_view_proj: Mat4,
) -> ([[f32; 4]; 4], [f32; 4]) {
    let Some(previous_camera) = frame.previous_motion_vector_camera() else {
        return (
            render_mat4_or(fallback_view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            [0.0, 0.0, 0.0, 0.0],
        );
    };
    let params =
        VelocityCameraParams::from_cameras(frame.viewport_size, camera, previous_camera, true);
    if !params.is_enabled() {
        return (
            render_mat4_or(fallback_view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            [0.0, 0.0, 0.0, 0.0],
        );
    }

    (params.previous_clip_from_world(), [1.0, 0.0, 0.0, 0.0])
}

fn jitter_params(camera: &crate::core::framework::render::ViewportCameraSnapshot) -> [f32; 4] {
    [
        camera.temporal_jitter.offset_pixels.x,
        camera.temporal_jitter.offset_pixels.y,
        camera.temporal_jitter.sequence_index as f32,
        if camera.temporal_jitter.sequence_index > 0 {
            1.0
        } else {
            0.0
        },
    ]
}

fn authored_ambient_color(
    frame: &crate::graphics::types::ViewportRenderFrame,
    fallback: RenderVec3,
) -> [f32; 4] {
    if frame.ambient_lights().is_empty() {
        return fallback.extend(1.0).to_array();
    }

    frame
        .ambient_lights()
        .iter()
        .fold(RenderVec3::ZERO, |accumulated, light| {
            accumulated + render_vec3_or(light.color * light.intensity, RenderVec3::ZERO)
        })
        .extend(1.0)
        .to_array()
}

#[cfg(test)]
mod tests {
    use super::SceneUniform;
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderAmbientLightSnapshot,
        RenderFrameExtract, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, TemporalJitterSample, ViewProjectionMatrixPair,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
    use crate::graphics::types::ViewportRenderFrame;

    #[test]
    fn scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.post_process.preview.lighting_enabled = true;
        extract
            .lighting
            .ambient_lights
            .push(RenderAmbientLightSnapshot {
                color: Vec3::new(0.05, 0.06, 0.07),
                intensity: 0.35,
                renderer_degraded: false,
                degradation_reason: None,
            });
        extract
            .lighting
            .ambient_lights
            .push(RenderAmbientLightSnapshot {
                color: Vec3::new(0.01, 0.02, 0.03),
                intensity: 0.5,
                renderer_degraded: false,
                degradation_reason: None,
            });
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        let uniform = SceneUniform::from_frame(&frame);

        assert_close(uniform.ambient_color[0], 0.0225);
        assert_close(uniform.ambient_color[1], 0.031);
        assert_close(uniform.ambient_color[2], 0.0395);
        assert_eq!(uniform.ambient_color[3], 1.0);
    }

    #[test]
    fn scene_uniform_appends_previous_view_projection_for_object_motion_vectors() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.view.camera.transform = Transform::from_translation(Vec3::new(0.5, 0.0, 0.0));
        let previous_camera = ViewportCameraSnapshot::default();
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
            .with_previous_motion_vector_camera(Some(previous_camera));

        let uniform = SceneUniform::from_frame(&frame);

        assert_eq!(uniform.motion_params, [1.0, 0.0, 0.0, 0.0]);
        assert_ne!(
            uniform.previous_view_proj_unjittered,
            uniform.view_proj_unjittered
        );
    }

    #[test]
    fn scene_uniform_exposes_jittered_and_unjittered_current_matrices() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.view.camera.temporal_jitter = TemporalJitterSample {
            offset_pixels: Vec2::new(0.5, -0.25),
            sequence_index: 3,
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(100, 50));

        let uniform = SceneUniform::from_frame(&frame);

        assert_ne!(uniform.view_proj, uniform.view_proj_unjittered);
        assert_eq!(
            uniform.previous_view_proj_unjittered,
            uniform.view_proj_unjittered
        );
        assert_eq!(uniform.jitter_params, [0.5, -0.25, 3.0, 1.0]);
    }

    #[test]
    fn scene_uniform_inverse_view_projection_is_unjittered() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.view.camera.temporal_jitter = TemporalJitterSample {
            offset_pixels: Vec2::new(0.5, -0.25),
            sequence_index: 3,
        };
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(100, 50));

        let uniform = SceneUniform::from_frame(&frame);
        let matrix_pair =
            ViewProjectionMatrixPair::from_camera(frame.camera(), UVec2::new(100, 50));

        assert_eq!(
            uniform.inverse_view_proj,
            matrix_pair
                .clip_from_world_unjittered
                .inverse()
                .to_cols_array_2d()
        );
        assert_ne!(
            uniform.inverse_view_proj,
            matrix_pair
                .clip_from_world_jittered
                .inverse()
                .to_cols_array_2d()
        );
    }

    fn empty_scene_snapshot() -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    projection_mode: ProjectionMode::Perspective,
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
