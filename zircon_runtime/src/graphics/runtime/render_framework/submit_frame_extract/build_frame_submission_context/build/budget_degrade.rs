use crate::core::framework::render::{
    PostProcessEffectKind, RenderDynamicResolutionSettings, RenderFrameExtract,
    RenderPostProcessEffectStackSettings,
};
use crate::graphics::{BuiltinRenderFeature, RenderPipelineCompileOptions};

use super::super::super::super::budget::BudgetDegradeSettings;

pub(super) fn apply_budget_render_scale(
    extract: &mut RenderFrameExtract,
    settings: BudgetDegradeSettings,
) {
    let authored_scale = extract.view.camera.dynamic_resolution.clamped_scale();
    let effective_scale = authored_scale.min(settings.render_scale);
    extract.view.camera.dynamic_resolution = if effective_scale < 1.0 {
        RenderDynamicResolutionSettings::fixed_scale(effective_scale)
    } else {
        RenderDynamicResolutionSettings::disabled()
    };
}

pub(super) fn compile_options_for_budget_degrade(
    mut options: RenderPipelineCompileOptions,
    settings: BudgetDegradeSettings,
) -> RenderPipelineCompileOptions {
    if settings.disable_ssao {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
            .with_plugin_feature_disabled("screen_space_ambient_occlusion");
    }
    if settings.disable_contact_shadow {
        options = options.with_plugin_feature_disabled("contact_shadow");
    }
    if settings.disable_bloom_high {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::Bloom)
            .with_post_process_effect_disabled(PostProcessEffectKind::Bloom);
    }
    options
}

pub(super) fn effect_stack_for_budget_degrade(
    mut effect_stack: RenderPostProcessEffectStackSettings,
    settings: BudgetDegradeSettings,
) -> RenderPostProcessEffectStackSettings {
    effect_stack.screen_space_reflection.roughness_mip_bias += settings.global_mip_bias as f32;
    if settings.disable_ssr {
        effect_stack.screen_space_reflection.intensity = 0.0;
    }
    effect_stack
}
