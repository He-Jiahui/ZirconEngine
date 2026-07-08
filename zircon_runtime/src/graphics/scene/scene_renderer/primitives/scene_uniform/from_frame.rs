use crate::core::framework::render::{ProjectionMode, SkyboxMode, ViewProjectionMatrixPair};
use crate::core::math::{Mat4, RenderMat4, RenderVec3};

use crate::graphics::scene::scene_renderer::temporal::velocity::velocity_camera_params::VelocityCameraParams;
use crate::graphics::types::ViewportRenderFrame;

use super::super::fallback::{render_mat4_or, render_vec3_or, render_vec4_or};
use super::SceneUniform;

impl SceneUniform {
    pub(crate) fn from_frame(frame: &ViewportRenderFrame) -> Self {
        let camera = frame.effective_camera();
        let ambient_color = if frame.preview().lighting_enabled {
            authored_ambient_color(frame, RenderVec3::splat(0.2))
        } else {
            RenderVec3::splat(0.55).extend(1.0).to_array()
        };

        let matrix_pair =
            ViewProjectionMatrixPair::from_camera(&camera, frame.render_region().local_size());
        let view_proj = matrix_pair.clip_from_world_jittered;
        let view_proj_unjittered = matrix_pair.clip_from_world_unjittered;
        let (previous_view_proj_unjittered, motion_params) =
            previous_motion_view_projection(frame, &camera, view_proj_unjittered);
        let skybox = &frame.environment().skybox;
        let sky_params = skybox.procedural;
        let source_cubemap_environment = skybox.source_cubemap_environment();
        let environment_sample_params = match skybox.mode {
            SkyboxMode::Disabled | SkyboxMode::ProceduralGradient => {
                [skybox.mode as u32 as f32, 0.0, 0.0, 0.0]
            }
            SkyboxMode::SourceCubemap => source_cubemap_environment
                .map(|environment| {
                    [
                        skybox.mode as u32 as f32,
                        environment.mip_chain.face_size() as f32,
                        environment.mip_chain.face_size() as f32,
                        environment.mip_chain.mip_count() as f32,
                    ]
                })
                .unwrap_or([skybox.mode as u32 as f32, 0.0, 0.0, 0.0]),
        };
        let environment_sh9 = source_cubemap_environment
            .map(|environment| environment.irradiance_sh9)
            .unwrap_or([[0.0; 4]; 9]);

        Self {
            view_proj: render_mat4_or(view_proj, RenderMat4::IDENTITY).to_cols_array_2d(),
            view_proj_unjittered: render_mat4_or(view_proj_unjittered, RenderMat4::IDENTITY)
                .to_cols_array_2d(),
            inverse_view_proj: render_mat4_or(view_proj_unjittered.inverse(), RenderMat4::IDENTITY)
                .to_cols_array_2d(),
            ambient_color,
            previous_view_proj_unjittered,
            motion_params,
            jitter_params: jitter_params(&camera),
            camera_world_position: render_vec3_or(camera.transform.translation, RenderVec3::ZERO)
                .extend(1.0)
                .to_array(),
            camera_view_direction: camera_view_direction(&camera),
            sky_horizon_color: render_vec4_or(
                sky_params.horizon_color,
                crate::core::math::Vec4::ZERO,
            )
            .to_array(),
            sky_zenith_color: render_vec4_or(
                sky_params.zenith_color,
                crate::core::math::Vec4::ZERO,
            )
            .to_array(),
            sky_ground_color: render_vec4_or(
                sky_params.ground_color,
                crate::core::math::Vec4::ZERO,
            )
            .to_array(),
            environment_params: [
                if skybox.is_enabled() { 1.0 } else { 0.0 },
                skybox.intensity().max(0.0),
                skybox.rotation_radians(),
                if frame.environment().ibl_bake_key().is_some() {
                    1.0
                } else {
                    0.0
                },
            ],
            environment_sample_params,
            environment_sh9,
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

fn camera_view_direction(
    camera: &crate::core::framework::render::ViewportCameraSnapshot,
) -> [f32; 4] {
    let view_direction = render_vec3_or(
        camera.transform.rotation * crate::core::math::Vec3::Z,
        RenderVec3::Z,
    )
    .normalize_or_zero();
    [
        view_direction.x,
        view_direction.y,
        view_direction.z,
        match camera.projection_mode {
            ProjectionMode::Orthographic => 1.0,
            ProjectionMode::Perspective => 0.0,
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
        EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
        RenderAmbientLightSnapshot, RenderFrameExtract, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        TemporalJitterSample, ViewProjectionMatrixPair, ViewportCameraSnapshot,
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
    fn scene_uniform_exports_camera_world_position() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.view.camera.transform = Transform::from_translation(Vec3::new(1.25, -2.5, 7.0));
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        let uniform = SceneUniform::from_frame(&frame);

        assert_eq!(uniform.camera_world_position, [1.25, -2.5, 7.0, 1.0]);
        assert_eq!(uniform.camera_view_direction, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn scene_uniform_marks_orthographic_camera_view_direction() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.view.camera.projection_mode = ProjectionMode::Orthographic;
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        let uniform = SceneUniform::from_frame(&frame);

        assert_eq!(uniform.camera_view_direction, [0.0, 0.0, 1.0, 1.0]);
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
        let camera = frame.effective_camera();
        let matrix_pair = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(100, 50));

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

    #[test]
    fn scene_uniform_uses_frame_render_region_size_for_projection_aspect() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.view.camera.projection_mode = ProjectionMode::Orthographic;
        extract.view.camera.ortho_size = 1.0;

        assert_eq!(extract.view.effective_render_size(), UVec2::new(1, 1));
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 960));

        let uniform = SceneUniform::from_frame(&frame);
        let camera = frame.effective_camera();
        let expected = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(1280, 960))
            .clip_from_world_unjittered
            .to_cols_array_2d();

        assert_eq!(uniform.view_proj_unjittered, expected);
        assert_close(uniform.view_proj_unjittered[0][0], 0.75);
        assert_close(uniform.view_proj_unjittered[1][1], 1.0);
    }

    #[test]
    fn scene_uniform_exports_environment_sky_parameters() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.environment = EnvironmentExtract::procedural_default();
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        let uniform = SceneUniform::from_frame(&frame);

        assert_eq!(uniform.sky_horizon_color, [0.16, 0.19, 0.24, 1.0]);
        assert_eq!(uniform.sky_zenith_color, [0.36, 0.46, 0.63, 1.0]);
        assert_eq!(uniform.sky_ground_color, [0.09, 0.11, 0.14, 1.0]);
        assert_eq!(uniform.environment_params, [1.0, 1.0, 0.0, 1.0]);
        assert_eq!(uniform.environment_sample_params, [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(uniform.environment_sh9, [[0.0; 4]; 9]);
    }

    #[test]
    fn scene_uniform_exports_source_cubemap_environment() {
        let mut source = crate::core::framework::render::SourceCubemapEnvironment::new(
            crate::core::framework::render::build_source_cubemap_from_equirect(4, |_, _| {
                [0.25, 0.5, 0.75, 1.0]
            }),
            9,
            [1, 2, 3, 4],
        );
        source.intensity = 1.75;
        source.rotation_radians = 0.5;
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            empty_scene_snapshot(),
        );
        extract.environment = EnvironmentExtract::source_cubemap(source);
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

        let uniform = SceneUniform::from_frame(&frame);

        assert_eq!(uniform.environment_params, [1.0, 1.75, 0.5, 1.0]);
        assert_eq!(uniform.environment_sample_params, [3.0, 4.0, 4.0, 3.0]);
        assert!(
            uniform.environment_sh9[0][0] > 0.0,
            "source cubemap should publish nonzero SH9 diffuse coefficients"
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
            environment: crate::core::framework::render::EnvironmentExtract::default(),
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
