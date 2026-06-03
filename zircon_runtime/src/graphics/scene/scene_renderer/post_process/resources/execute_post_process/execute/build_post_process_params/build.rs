use crate::core::framework::render::{
    AntiAliasMode, ProjectionMode, RenderFrameExtract, RenderPostProcessEffectStackSettings,
    RenderTonemapOperator,
};
use crate::core::math::UVec2;

use super::super::super::super::super::post_process_params::PostProcessParams;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::baked_lighting::baked_lighting;
use super::color_grading::color_grading;

pub(in super::super) fn build_post_process_params(
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
        effect_vignette_grain: effect_vignette_grain(effect_stack),
        effect_chromatic_fog: effect_chromatic_fog(effect_stack),
        effect_fog_color: effect_fog_color(effect_stack),
        effect_dither_ssr: effect_dither_ssr(effect_stack),
        effect_ssr_limits: effect_ssr_limits(effect_stack),
        effect_depth: effect_depth(extract),
    }
}

fn effect_flags(settings: RenderPostProcessEffectStackSettings) -> [u32; 4] {
    [
        tonemap_operator_id(settings.tonemap.operator),
        u32::from(settings.color_lookup.is_enabled()),
        settings.screen_space_reflection.max_steps,
        u32::from(settings.is_enabled()),
    ]
}

fn tonemap_operator_id(operator: RenderTonemapOperator) -> u32 {
    match operator {
        RenderTonemapOperator::None => 0,
        RenderTonemapOperator::Reinhard => 1,
        RenderTonemapOperator::Aces => 2,
        RenderTonemapOperator::Filmic => 3,
    }
}

fn effect_tonemap_lut(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.tonemap.exposure_bias,
        settings.tonemap.white_point.max(0.001),
        settings.color_lookup.intensity.max(0.0),
        0.0,
    ]
}

fn effect_blur_dof(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.blur.radius.max(0.0),
        settings.depth_of_field.focus_distance.max(0.0),
        settings.depth_of_field.aperture.max(0.0),
        settings.depth_of_field.max_blur_radius.max(0.0),
    ]
}

fn effect_vignette_grain(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.vignette.intensity.max(0.0),
        settings.vignette.smoothness.max(0.001),
        settings.vignette.roundness.max(0.001),
        settings.grain.intensity.max(0.0),
    ]
}

fn effect_chromatic_fog(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.chromatic_aberration.intensity.max(0.0),
        settings.chromatic_aberration.sample_spread.max(0.0),
        settings.fog.density.max(0.0),
        settings.fog.height_falloff.max(0.0),
    ]
}

fn effect_fog_color(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.fog.color.x.max(0.0),
        settings.fog.color.y.max(0.0),
        settings.fog.color.z.max(0.0),
        settings.grain.response.max(0.0),
    ]
}

fn effect_dither_ssr(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.dither.intensity.max(0.0),
        settings.dither.scale.max(0.001),
        settings.screen_space_reflection.intensity.max(0.0),
        settings.screen_space_reflection.thickness.max(0.0),
    ]
}

fn effect_ssr_limits(settings: RenderPostProcessEffectStackSettings) -> [f32; 4] {
    [
        settings.screen_space_reflection.max_ray_distance.max(0.0),
        settings.screen_space_reflection.max_steps as f32,
        0.0,
        0.0,
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

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderDepthOfFieldSettings, RenderDitherSettings, RenderFogSettings, RenderFrameExtract,
        RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
        RenderTonemapOperator, RenderTonemapSettings, RenderWorldSnapshotHandle,
    };
    use crate::core::math::UVec2;
    use crate::core::math::Vec3;
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
                aperture: 0.6,
                max_blur_radius: 3.0,
            },
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 24,
                ..Default::default()
            },
            dither: RenderDitherSettings {
                intensity: 0.1,
                scale: 2.0,
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
        assert_near(params.effect_chromatic_fog[2], 0.2);
        assert_near(params.effect_fog_color[2], 0.5);
        assert_near(params.effect_dither_ssr[0], 0.1);
        assert_near(params.effect_dither_ssr[2], 0.5);
        assert_near(params.effect_ssr_limits[1], 24.0);
        assert_near(params.effect_depth[0], 0.25);
        assert_near(params.effect_depth[1], 128.0);
        assert_near(params.effect_depth[2], 1.0 / 127.75);
        assert_near(params.effect_depth[3], 1.0);
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
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "expected {actual} to be near {expected}"
        );
    }
}
