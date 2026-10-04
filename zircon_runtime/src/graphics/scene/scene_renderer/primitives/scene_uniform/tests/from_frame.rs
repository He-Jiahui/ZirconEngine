use super::{hit_proxy_clip_crop, SceneUniform};
use crate::core::framework::render::{
    EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
    RenderAmbientLightSnapshot, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    TemporalJitterSample, ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::{Quat, Transform, UVec2, Vec2, Vec3, Vec4};
use crate::graphics::scene::scene_renderer::primitives::SceneEnvironmentSh9;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::ViewportRenderRegion;

#[test]
fn hit_proxy_clip_crop_maps_the_requested_pixel_center_to_the_single_texel() {
    let region = ViewportRenderRegion::full_target(UVec2::new(100, 50));
    let pixel = UVec2::new(73, 11);
    let crop = hit_proxy_clip_crop(pixel, region).expect("pixel inside render region");
    let center_x = 2.0 * (pixel.x as f32 + 0.5) / 100.0 - 1.0;
    let center_y = 1.0 - 2.0 * (pixel.y as f32 + 0.5) / 50.0;

    let cropped = crop * Vec4::new(center_x, center_y, 0.25, 1.0);

    assert_close(cropped.x, 0.0);
    assert_close(cropped.y, 0.0);
    assert_close(cropped.z, 0.25);
    assert_close(cropped.w, 1.0);
    assert!(hit_proxy_clip_crop(UVec2::new(100, 49), region).is_none());
}

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
            affects_lightmapped_meshes: false,
            renderer_degraded: false,
            degradation_reason: None,
        });
    extract
        .lighting
        .ambient_lights
        .push(RenderAmbientLightSnapshot {
            color: Vec3::new(0.01, 0.02, 0.03),
            intensity: 0.5,
            affects_lightmapped_meshes: true,
            renderer_degraded: false,
            degradation_reason: None,
        });
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let uniform = SceneUniform::from_frame(&frame);

    assert_close(uniform.ambient_color[0], 0.0225);
    assert_close(uniform.ambient_color[1], 0.031);
    assert_close(uniform.ambient_color[2], 0.0395);
    assert_eq!(uniform.ambient_color[3], 1.0);
    assert_close(uniform.lightmapped_ambient_color[0], 0.005);
    assert_close(uniform.lightmapped_ambient_color[1], 0.01);
    assert_close(uniform.lightmapped_ambient_color[2], 0.015);
    assert_eq!(uniform.lightmapped_ambient_color[3], 1.0);
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

    assert_eq!(uniform.camera_world_position, [1.25, -2.5, 7.0, 0.0]);
    assert_eq!(uniform.camera_view_direction, [0.0, 0.0, 1.0, 0.0]);
}

#[test]
fn scene_uniform_orthographic_view_direction_uses_rotated_camera_backward_axis() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        empty_scene_snapshot(),
    );
    extract.view.camera.projection_mode = ProjectionMode::Orthographic;
    extract.view.camera.transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let uniform = SceneUniform::from_frame(&frame);

    assert_close(uniform.camera_view_direction[0], 1.0);
    assert_close(uniform.camera_view_direction[1], 0.0);
    assert_close(uniform.camera_view_direction[2], 0.0);
    assert_eq!(uniform.camera_view_direction[3], 1.0);
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
    extract.environment = EnvironmentExtract::procedural_default().into();
    extract.environment.skybox.procedural.sun_direction = Vec4::new(0.0, 3.0, 4.0, 0.0);
    extract.environment.skybox.procedural.sun_intensity = 2.0;
    extract
        .environment
        .skybox
        .procedural
        .sun_angular_radius_radians = 0.08;
    extract.environment.skybox.procedural.rotation_radians = 0.5;
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let uniform = SceneUniform::from_frame(&frame);

    assert_eq!(uniform.sky_horizon_color, [0.16, 0.19, 0.24, 1.0]);
    assert_eq!(uniform.sky_zenith_color, [0.36, 0.46, 0.63, 1.0]);
    assert_eq!(uniform.sky_ground_color, [0.09, 0.11, 0.14, 1.0]);
    let rotation = 0.5_f32;
    let expected_sun_direction = Vec3::new(0.8 * rotation.sin(), 0.6, 0.8 * rotation.cos());
    assert_close(uniform.sky_sun_direction[0], expected_sun_direction.x);
    assert_close(uniform.sky_sun_direction[1], expected_sun_direction.y);
    assert_close(uniform.sky_sun_direction[2], expected_sun_direction.z);
    assert_eq!(uniform.sky_sun_direction[3], 1.0);
    assert_eq!(uniform.sky_sun_params[0], 2.0);
    assert_close(uniform.sky_sun_params[1], 0.08_f32.cos());
    assert_close(uniform.sky_sun_params[2], (0.08_f32 * 0.72).cos());
    assert_eq!(uniform.environment_params, [0.0, 1.0, 0.5, 1.0]);
    assert_close(uniform.environment_rotation_sin_cos[0], rotation.sin());
    assert_close(uniform.environment_rotation_sin_cos[1], rotation.cos());
    assert_eq!(uniform.environment_rotation_sin_cos[2], 1.0);
    assert_eq!(uniform.environment_sample_params, [1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn scene_uniform_realtime_ibl_override_selects_cube_pmrem_sampling() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        empty_scene_snapshot(),
    );
    extract.environment = EnvironmentExtract::procedural_default().into();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let mut uniform = SceneUniform::from_frame(&frame);
    uniform.use_realtime_ibl(128, 128, 8);

    assert_eq!(uniform.environment_params, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(uniform.environment_rotation_sin_cos, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(uniform.environment_sample_params, [4.0, 128.0, 128.0, 8.0]);
}

#[test]
fn scene_uniform_ibl_availability_does_not_hash_the_bake_identity() {
    let implementation = include_str!("../from_frame.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("scene uniform implementation");

    assert!(!implementation.contains("ibl_bake_key("));
    assert!(implementation.contains("let has_ibl_source = match skybox.mode"));
}

#[test]
fn scene_uniform_sanitizes_non_finite_environment_rotation() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        empty_scene_snapshot(),
    );
    extract.environment = EnvironmentExtract::procedural_default().into();
    extract.environment.skybox.procedural.sun_intensity = 1.0;
    extract.environment.skybox.procedural.rotation_radians = f32::NAN;
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let uniform = SceneUniform::from_frame(&frame);

    assert_eq!(uniform.environment_params[2], 0.0);
    assert_eq!(uniform.environment_rotation_sin_cos, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(uniform.sky_sun_direction, [0.0, 1.0, 0.0, 1.0]);
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
    extract.environment = EnvironmentExtract::source_cubemap(source).into();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let uniform = SceneUniform::from_frame(&frame);
    let environment_sh9 = SceneEnvironmentSh9::from_frame(&frame);

    assert_eq!(uniform.environment_params, [0.0, 1.75, 0.5, 1.0]);
    assert_close(uniform.environment_rotation_sin_cos[0], 0.5_f32.sin());
    assert_close(uniform.environment_rotation_sin_cos[1], 0.5_f32.cos());
    assert_eq!(uniform.environment_rotation_sin_cos[2], 1.0);
    assert_eq!(uniform.environment_sample_params, [3.0, 4.0, 128.0, 8.0]);
    assert!(
        environment_sh9.coefficients()[0][0] > 0.0,
        "source cubemap should publish nonzero SH9 diffuse coefficients"
    );
}

#[test]
fn scene_uniform_marks_source_cubemap_irradiance_cube_availability() {
    let source = crate::core::framework::render::SourceCubemapEnvironment::new(
        crate::core::framework::render::build_source_cubemap_from_equirect(4, |_, _| {
            [0.25, 0.5, 0.75, 1.0]
        }),
        9,
        [1, 2, 3, 4],
    )
    .with_irradiance_cube(
        crate::core::framework::render::SourceCubemapIrradianceCube::new(
            1,
            vec![[0.25, 0.5, 0.75]; 6],
        ),
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        empty_scene_snapshot(),
    );
    extract.environment = EnvironmentExtract::source_cubemap(source).into();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let uniform = SceneUniform::from_frame(&frame);

    assert_eq!(uniform.environment_params[0], 1.0);
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
