use crate::core::framework::render::{AntiAliasMode, RenderPostProcessEffectStackSettings};

use super::super::RenderPassExecutionContext;

fn frame_post_process_effect_stack(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<RenderPostProcessEffectStackSettings, String> {
    Ok(context
        .require_gpu()?
        .frame_extract()
        .post_process
        .effect_stack)
}

pub(super) fn frame_uses_scene_velocity(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<bool, String> {
    let gpu = context.require_gpu()?;
    let frame_extract = gpu.frame_extract();
    Ok(frame_extract.view.anti_alias.mode == AntiAliasMode::Taa
        || frame_extract
            .post_process
            .effect_stack
            .motion_blur
            .is_enabled()
        || frame_extract
            .post_process
            .effect_stack
            .screen_space_reflection
            .is_enabled())
}

pub(super) fn frame_uses_taa(context: &mut RenderPassExecutionContext<'_>) -> Result<bool, String> {
    Ok(context.require_gpu()?.frame_extract().view.anti_alias.mode == AntiAliasMode::Taa)
}

pub(super) fn frame_uses_reconstructed_motion_vectors(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<bool, String> {
    let effect_stack = frame_post_process_effect_stack(context)?;
    Ok(effect_stack.motion_blur.is_enabled() || effect_stack.screen_space_reflection.is_enabled())
}

pub(super) fn frame_uses_depth_of_field(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<bool, String> {
    Ok(frame_post_process_effect_stack(context)?
        .depth_of_field
        .is_enabled())
}

pub(super) fn frame_uses_screen_space_reflection(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<bool, String> {
    Ok(frame_post_process_effect_stack(context)?
        .screen_space_reflection
        .is_enabled())
}
