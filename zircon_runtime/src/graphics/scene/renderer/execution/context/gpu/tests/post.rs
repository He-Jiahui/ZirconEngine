use super::{
    color_lut_bake_dispatch_resource_accesses, effect_stack_uses_reconstructed_velocity,
    require_post_process_phase_targets, require_post_process_render_region,
};
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderMotionBlurSettings, RenderPipelinePhase,
    RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
};
use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderRegion;
use crate::render_graph::{RenderGraphResourceAccessKind, RenderGraphResourceKind};

#[test]
fn color_lut_bake_dispatch_reports_exposure_read_and_lut_write() {
    let accesses =
        color_lut_bake_dispatch_resource_accesses(PostProcessGraphResourceNames::COLOR_LUT);

    assert_eq!(accesses.len(), 2);
    assert_eq!(
        accesses[0].name,
        PostProcessGraphResourceNames::EXPOSURE_CURRENT
    );
    assert_eq!(accesses[0].kind, RenderGraphResourceKind::TransientBuffer);
    assert_eq!(accesses[0].access, RenderGraphResourceAccessKind::Read);
    assert_eq!(accesses[1].name, PostProcessGraphResourceNames::COLOR_LUT);
    assert_eq!(accesses[1].kind, RenderGraphResourceKind::TransientTexture);
    assert_eq!(accesses[1].access, RenderGraphResourceAccessKind::Write);
}

#[test]
fn reconstructed_velocity_is_requested_for_temporal_effects() {
    assert!(!effect_stack_uses_reconstructed_velocity(
        RenderPostProcessEffectStackSettings::default()
    ));

    assert!(effect_stack_uses_reconstructed_velocity(
        RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 4,
            },
            ..Default::default()
        }
    ));

    assert!(effect_stack_uses_reconstructed_velocity(
        RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 16,
                ..Default::default()
            },
            ..Default::default()
        }
    ));
}

#[test]
fn missing_post_process_phase_is_a_fallible_graph_error() {
    let phase = RenderPipelinePhase::DisplayPostProcess;
    let error = require_post_process_render_region("post.fxaa", "FXAA", phase, None)
        .expect_err("missing phase must fail without panicking");

    assert_eq!(
        error,
        "FXAA graph executor for pass `post.fxaa` requires phase DisplayPostProcess"
    );
    let region = ViewportRenderRegion::full_target(UVec2::new(640, 360));
    assert_eq!(
        require_post_process_render_region("post.fxaa", "FXAA", phase, Some(region)),
        Ok(region)
    );

    let target_error =
        require_post_process_phase_targets("post.hzb", RenderPipelinePhase::SceneLinear, None)
            .expect_err("missing phase targets must fail without panicking");
    assert_eq!(
        target_error,
        "post-process stack graph executor for pass `post.hzb` requires phase SceneLinear"
    );
}
