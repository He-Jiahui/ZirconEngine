use crate::core::framework::render::{
    AntiAliasMode, ProjectionMode, RenderFrameExtract, RenderPostProcessEffectStackSettings,
};
use crate::core::math::{UVec2, Vec3};

use super::super::super::super::super::post_process_params::PostProcessParams;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::baked_lighting::baked_lighting;
use super::color_grading::color_grading;

pub(in crate::graphics::scene::scene_renderer::post_process::resources) fn build_post_process_params(
    viewport_size: UVec2,
    cluster_dimensions: UVec2,
    extract: &RenderFrameExtract,
    features: SceneRuntimeFeatureFlags,
    history_available: bool,
    reflection_probe_count: u32,
    hybrid_gi_probe_count: u32,
    scheduled_trace_region_count: u32,
) -> PostProcessParams {
    let color_grading = color_grading(extract, features);
    let baked_lighting = baked_lighting(extract, features);
    let effect_stack = extract.post_process.effect_stack;
    let effect_view = effect_view_basis_rows(extract);

    PostProcessParams {
        viewport_and_clusters: [
            viewport_size.x.max(1),
            viewport_size.y.max(1),
            cluster_dimensions.x.max(1),
            cluster_dimensions.y.max(1),
        ],
        feature_flags: [
            u32::from(features.ssao_enabled),
            u32::from(features.clustered_lighting_enabled),
            u32::from(features.history_resolve_enabled && history_available),
            reflection_probe_count,
        ],
        hybrid_gi_counts: [
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
            u32::from(features.hybrid_global_illumination_enabled && history_available),
            0,
        ],
        anti_alias: [
            u32::from(
                features.anti_alias_enabled && extract.view.anti_alias.mode == AntiAliasMode::Fxaa,
            ),
            0,
            0,
            0,
        ],
        blends: [
            0.24,
            0.0,
            0.0,
            if features.bloom_enabled {
                extract.post_process.bloom.intensity.max(0.0)
            } else {
                0.0
            },
        ],
        grading: [
            color_grading.exposure.max(0.0),
            color_grading.contrast.max(0.0),
            color_grading.saturation.max(0.0),
            color_grading.gamma.max(0.001),
        ],
        tint_and_probe: [
            color_grading.tint.x.max(0.0),
            color_grading.tint.y.max(0.0),
            color_grading.tint.z.max(0.0),
            if features.reflection_probes_enabled {
                0.35
            } else {
                0.0
            },
        ],
        hybrid_gi_color_and_intensity: [
            0.32,
            0.38,
            0.46,
            if features.hybrid_global_illumination_enabled && hybrid_gi_probe_count > 0 {
                0.4
            } else {
                0.0
            },
        ],
        baked_color_and_intensity: [
            baked_lighting.color.x.max(0.0),
            baked_lighting.color.y.max(0.0),
            baked_lighting.color.z.max(0.0),
            baked_lighting.intensity.max(0.0),
        ],
        effect_flags: effect_flags(effect_stack),
        effect_tonemap_lut: effect_tonemap_lut(effect_stack),
        effect_blur_dof: effect_blur_dof(effect_stack),
        effect_dof_lens: effect_dof_lens(effect_stack),
        effect_vignette_grain: effect_vignette_grain(effect_stack),
        effect_chromatic_fog: effect_chromatic_fog(effect_stack),
        effect_fog_color: effect_fog_color(effect_stack),
        effect_dither_ssr: effect_dither_ssr(effect_stack),
        effect_ssr_limits: effect_ssr_limits(effect_stack),
        effect_depth: effect_depth(extract),
        effect_projection: effect_projection(viewport_size, extract),
        effect_view_x: effect_view[0],
        effect_view_y: effect_view[1],
        effect_view_z: effect_view[2],
        effect_motion_blur: effect_motion_blur(effect_stack),
    }
}

fn effect_flags(settings: RenderPostProcessEffectStackSettings) -> [u32; 4] {
    [
        settings.tonemap.render_operator_id(),
        u32::from(settings.color_lookup.is_enabled()),
        settings.screen_space_reflection.max_steps,
        u32::from(settings.is_enabled()),
    ]
}

fn effect_tonemap_lut(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.tonemap.render_exposure_bias(),
        settings.tonemap.render_white_point(),
        settings.color_lookup.render_intensity(),
        0.0,
    ]
}

fn effect_blur_dof(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.blur.render_radius(),
        settings.depth_of_field.render_focus_distance(),
        settings.depth_of_field.render_aperture(),
        settings.depth_of_field.render_max_blur_radius(),
    ]
}

fn effect_dof_lens(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.depth_of_field.render_focal_length_mm(),
        settings.depth_of_field.render_focus_range(),
        settings.depth_of_field.render_bokeh_blade_count() as f32,
        settings.depth_of_field.bokeh_rotation_radians,
    ]
}

fn effect_motion_blur(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.motion_blur.render_shutter_angle(),
        settings.motion_blur.render_samples() as f32,
        0.0,
        0.0,
    ]
}

fn effect_vignette_grain(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.vignette.render_intensity(),
        settings.vignette.render_smoothness(),
        settings.vignette.render_roundness(),
        settings.grain.render_intensity(),
    ]
}

fn effect_chromatic_fog(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.chromatic_aberration.render_intensity(),
        settings.chromatic_aberration.render_sample_spread(),
        settings.fog.render_density(),
        settings.fog.render_height_falloff(),
    ]
}

fn effect_fog_color(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    let fog_color = settings.fog.render_color();

    [
        fog_color.x,
        fog_color.y,
        fog_color.z,
        settings.grain.render_response(),
    ]
}

fn effect_dither_ssr(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.dither.render_intensity(),
        settings.dither.render_scale(),
        settings.screen_space_reflection.render_intensity(),
        settings.screen_space_reflection.render_thickness(),
    ]
}

fn effect_ssr_limits(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.screen_space_reflection.render_max_ray_distance(),
        settings.screen_space_reflection.max_steps as f32,
        settings
            .screen_space_reflection
            .render_temporal_blend_factor(),
        settings.screen_space_reflection.render_roughness_mip_bias(),
    ]
}

fn effect_depth(extract: &RenderFrameExtract) -> [f32; 4] {
    let camera = &extract.view.camera;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near + 0.001);

    [
        near,
        far,
        1.0 / (far - near).max(0.001),
        if matches!(camera.projection_mode, ProjectionMode::Perspective) {
            1.0
        } else {
            0.0
        },
    ]
}

fn effect_projection(viewport_size: UVec2, extract: &RenderFrameExtract) -> [f32; 4] {
    let camera = &extract.view.camera;
    let aspect = viewport_size.x.max(1) as f32 / viewport_size.y.max(1) as f32;
    let fov_y = camera
        .fov_y_radians
        .clamp(0.001, std::f32::consts::PI - 0.001);
    let focal_y = 1.0 / (fov_y * 0.5).tan().max(0.001);
    let focal_x = focal_y / aspect.max(0.001);
    let half_height = camera.ortho_size.max(0.01);
    let half_width = half_height * aspect.max(0.001);

    [focal_x, focal_y, half_width, half_height]
}

fn effect_view_basis_rows(extract: &RenderFrameExtract) -> [[f32; 4]; 3] {
    let transform = extract.view.camera.transform;

    [
        camera_axis_row(transform.right(), Vec3::X),
        camera_axis_row(transform.up(), Vec3::Y),
        camera_axis_row((transform.rotation * Vec3::Z).normalize_or_zero(), Vec3::Z),
    ]
}

fn camera_axis_row(axis: Vec3, fallback: Vec3) -> [f32; 4] {
    let normalized = if axis.is_finite() && axis.length_squared() > 0.0001 {
        axis.normalize()
    } else {
        fallback
    };

    normalized.extend(0.0).to_array()
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderChromaticAberrationSettings, RenderDepthOfFieldSettings, RenderDitherSettings,
        RenderFilmGrainSettings, RenderFogSettings, RenderFrameExtract, RenderMotionBlurSettings,
        RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
        RenderTonemapOperator, RenderTonemapSettings, RenderVignetteSettings,
        RenderWorldSnapshotHandle,
    };
    use crate::core::math::{Transform, UVec2, Vec3};
    use crate::scene::World;

    use super::*;

    #[test]
    fn clustered_lighting_does_not_tint_final_frame_by_tile_buffer() {
        let params = build_post_process_params(
            UVec2::new(128, 96),
            UVec2::new(8, 6),
            &RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(1),
                World::new().to_render_snapshot(),
            ),
            SceneRuntimeFeatureFlags {
                clustered_lighting_enabled: true,
                ..SceneRuntimeFeatureFlags::default()
            },
            false,
            0,
            0,
            0,
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
            &extract,
            SceneRuntimeFeatureFlags::default(),
            false,
            0,
            0,
            0,
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
            &extract,
            SceneRuntimeFeatureFlags::default(),
            false,
            0,
            0,
            0,
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
            &extract,
            SceneRuntimeFeatureFlags::default(),
            false,
            0,
            0,
            0,
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
}
