use crate::core::framework::render::{
    ProjectionMode, RenderChromaticAberrationSettings, RenderDepthOfFieldSettings,
    RenderDitherSettings, RenderFilmGrainSettings, RenderFogSettings, RenderFrameExtract,
    RenderMotionBlurSettings, RenderPostProcessEffectStackSettings,
    RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
    RenderVignetteSettings, RenderWorldSnapshotHandle, TemporalJitterSample,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3};
use crate::scene::World;

use super::*;

#[test]
fn clustered_lighting_does_not_tint_final_frame_by_tile_buffer() {
    let params = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        &PostProcessExtract::default(),
        SceneRuntimeFeatureFlags {
            clustered_lighting_enabled: true,
            ..SceneRuntimeFeatureFlags::default()
        },
        false,
        0,
        0,
        0,
        false,
    );

    assert_eq!(
        params.blends[1], 0.0,
        "cluster buffer intensity must not create visible viewport tile bands"
    );
    assert_eq!(
        params.blends[2], 0.0,
        "cluster buffer color must not create visible viewport tile bands"
    );
}

#[test]
fn contact_shadow_runtime_flag_is_encoded_separately_from_ssao() {
    let params = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        &PostProcessExtract::default(),
        SceneRuntimeFeatureFlags {
            contact_shadow_enabled: true,
            ..SceneRuntimeFeatureFlags::default()
        },
        false,
        0,
        0,
        0,
        false,
    );

    assert_eq!(
        params.feature_flags[0], 0,
        "contact shadows must not masquerade as SSAO"
    );
    assert_eq!(params.lighting_flags[0], 1);
}

#[test]
fn hybrid_gi_current_lighting_flag_is_encoded_separately_from_history() {
    let params = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        &PostProcessExtract::default(),
        SceneRuntimeFeatureFlags {
            hybrid_global_illumination_enabled: true,
            ..SceneRuntimeFeatureFlags::default()
        },
        false,
        0,
        3,
        2,
        true,
    );

    assert_eq!(params.hybrid_gi_counts, [3, 2, 0, 1]);
}

#[test]
fn post_process_params_pack_physical_viewport_and_local_scene_source_origins_separately() {
    let mut camera = crate::core::framework::render::CameraRenderDescriptor::from_camera_payload(
        None,
        crate::core::framework::render::ViewportCameraSnapshot::default(),
    );
    camera.viewport_rect = Some(crate::core::framework::render::RenderViewportRect::new(
        UVec2::new(320, 40),
        UVec2::new(320, 180),
    ));
    let region = ViewportRenderRegion::from_camera(Some(&camera), UVec2::new(640, 360));

    let params = build_post_process_params(
        UVec2::new(320, 180),
        UVec2::new(20, 12),
        region,
        [0, 0],
        &RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        ),
        &PostProcessExtract::default(),
        SceneRuntimeFeatureFlags::default(),
        false,
        0,
        0,
        0,
        false,
    );

    assert_eq!(params.viewport_and_clusters, [320, 180, 320, 40]);
    assert_eq!(params.cluster_dimensions, [20, 12, 0, 0]);
}

#[test]
fn effect_stack_settings_are_encoded_into_post_process_params() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: RenderTonemapOperator::Aces,
            exposure_bias: 0.25,
            white_point: 1.4,
        },
        depth_of_field: RenderDepthOfFieldSettings {
            focus_distance: 8.0,
            focus_range: 2.5,
            aperture: 0.6,
            focal_length_mm: 85.0,
            max_blur_radius: 3.0,
            bokeh_blade_count: 7,
            bokeh_rotation_radians: 0.35,
        },
        motion_blur: RenderMotionBlurSettings {
            shutter_angle: 0.5,
            samples: 12,
        },
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 0.5,
            max_steps: 24,
            temporal_blend_factor: 0.27,
            roughness_mip_bias: 0.35,
            ..Default::default()
        },
        vignette: RenderVignetteSettings {
            intensity: 0.35,
            smoothness: 0.65,
            roundness: 0.8,
        },
        grain: RenderFilmGrainSettings {
            intensity: 0.07,
            response: 0.9,
        },
        dither: RenderDitherSettings {
            intensity: 0.1,
            scale: 2.0,
        },
        chromatic_aberration: RenderChromaticAberrationSettings {
            intensity: 0.12,
            sample_spread: 1.75,
        },
        fog: RenderFogSettings {
            density: 0.2,
            height_falloff: 0.4,
            color: Vec3::new(0.3, 0.4, 0.5),
        },
        ..Default::default()
    };
    extract.view.camera.z_near = 0.25;
    extract.view.camera.z_far = 128.0;
    extract.view.camera.fov_y_radians = std::f32::consts::FRAC_PI_2;
    extract.view.camera.projection_mode =
        crate::core::framework::render::ProjectionMode::Perspective;

    let params = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &extract,
        &extract.post_process,
        SceneRuntimeFeatureFlags::default(),
        false,
        0,
        0,
        0,
        false,
    );

    assert_eq!(params.effect_flags[0], 2);
    assert_eq!(params.effect_flags[2], 24);
    assert_eq!(params.effect_flags[3], 1);
    assert_near(params.effect_tonemap_lut[0], 0.25);
    assert_near(params.effect_blur_dof[2], 0.6);
    assert_near(params.effect_dof_lens[0], 85.0);
    assert_near(params.effect_dof_lens[1], 2.5);
    assert_near(params.effect_dof_lens[2], 7.0);
    assert_near(params.effect_dof_lens[3], 0.35);
    assert_near(params.effect_motion_blur[0], 0.5);
    assert_near(params.effect_motion_blur[1], 12.0);
    assert_near(params.effect_vignette_grain[0], 0.35);
    assert_near(params.effect_vignette_grain[1], 0.65);
    assert_near(params.effect_vignette_grain[2], 0.8);
    assert_near(params.effect_vignette_grain[3], 0.07);
    assert_near(params.effect_chromatic_fog[0], 0.12);
    assert_near(params.effect_chromatic_fog[1], 1.75);
    assert_near(params.effect_chromatic_fog[2], 0.2);
    assert_near(params.effect_chromatic_fog[3], 0.4);
    assert_near(params.effect_fog_color[0], 0.3);
    assert_near(params.effect_fog_color[1], 0.4);
    assert_near(params.effect_fog_color[2], 0.5);
    assert_near(params.effect_fog_color[3], 0.9);
    assert_near(params.effect_dither_ssr[0], 0.1);
    assert_near(params.effect_dither_ssr[1], 2.0);
    assert_near(params.effect_dither_ssr[2], 0.5);
    assert_near(params.effect_dither_ssr[3], 0.1);
    assert_near(params.effect_ssr_limits[1], 24.0);
    assert_near(params.effect_ssr_limits[2], 0.27);
    assert_near(params.effect_ssr_limits[3], 0.35);
    assert_near(params.effect_depth[0], 0.25);
    assert_near(params.effect_depth[1], 128.0);
    assert_near(params.effect_depth[2], 1.0 / 127.75);
    assert_near(params.effect_depth[3], 1.0);
    assert_near(params.effect_projection[0], 0.75);
    assert_near(params.effect_projection[1], 1.0);
}

#[test]
fn camera_view_basis_is_encoded_for_post_process_normals() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.view.camera.transform = Transform::looking_at(Vec3::ZERO, -Vec3::X, Vec3::Y);

    let params = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &extract,
        &extract.post_process,
        SceneRuntimeFeatureFlags::default(),
        false,
        0,
        0,
        0,
        false,
    );

    assert_near(params.effect_view_x[0], 0.0);
    assert_near(params.effect_view_x[1], 0.0);
    assert_near(params.effect_view_x[2], -1.0);
    assert_near(params.effect_view_x[3], 0.0);
    assert_near(params.effect_view_y[0], 0.0);
    assert_near(params.effect_view_y[1], 1.0);
    assert_near(params.effect_view_y[2], 0.0);
    assert_near(params.effect_view_y[3], 0.0);
    assert_near(params.effect_view_z[0], 1.0);
    assert_near(params.effect_view_z[1], 0.0);
    assert_near(params.effect_view_z[2], 0.0);
    assert_near(params.effect_view_z[3], 0.0);
}

#[test]
fn post_process_projection_params_ignore_temporal_jitter() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.view.camera.fov_y_radians = std::f32::consts::FRAC_PI_2;
    extract.view.camera.projection_mode = ProjectionMode::Perspective;

    let unjittered = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &extract,
        &extract.post_process,
        SceneRuntimeFeatureFlags::default(),
        false,
        0,
        0,
        0,
        false,
    );

    extract.view.camera.temporal_jitter = TemporalJitterSample {
        offset_pixels: Vec2::new(0.5, -0.25),
        sequence_index: 7,
    };
    let jittered = build_post_process_params(
        UVec2::new(128, 96),
        UVec2::new(8, 6),
        ViewportRenderRegion::full_target(UVec2::new(128, 96)),
        [0, 0],
        &extract,
        &extract.post_process,
        SceneRuntimeFeatureFlags::default(),
        false,
        0,
        0,
        0,
        false,
    );

    assert_eq!(jittered.effect_projection, unjittered.effect_projection);
    assert_eq!(jittered.effect_view_x, unjittered.effect_view_x);
    assert_eq!(jittered.effect_view_y, unjittered.effect_view_y);
    assert_eq!(jittered.effect_view_z, unjittered.effect_view_z);
}

#[test]
fn orthographic_camera_depth_params_disable_perspective_linearization() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.view.camera.projection_mode =
        crate::core::framework::render::ProjectionMode::Orthographic;
    extract.view.camera.z_near = -4.0;
    extract.view.camera.z_far = -4.0;
    extract.view.camera.ortho_size = 4.0;

    let params = build_post_process_params(
        UVec2::new(64, 64),
        UVec2::new(4, 4),
        ViewportRenderRegion::full_target(UVec2::new(64, 64)),
        [0, 0],
        &extract,
        &extract.post_process,
        SceneRuntimeFeatureFlags::default(),
        false,
        0,
        0,
        0,
        false,
    );

    assert_near(params.effect_depth[0], 0.001);
    assert_near(params.effect_depth[1], 0.002);
    assert_near(params.effect_depth[2], 1000.0);
    assert_near(params.effect_depth[3], 0.0);
    assert_near(params.effect_projection[2], 4.0);
    assert_near(params.effect_projection[3], 4.0);
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "expected {actual} to be near {expected}"
    );
}
