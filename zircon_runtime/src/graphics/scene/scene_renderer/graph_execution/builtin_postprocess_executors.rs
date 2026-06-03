use crate::core::framework::render::{PostProcessEffectKind, PostProcessGraphResourceNames};

use super::RenderPassExecutionContext;

pub(super) fn bloom_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Bloom)
}

pub(super) fn color_grading_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::ColorGrading)
}

pub(super) fn history_resolve_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::HistoryResolve)
}

pub(super) fn effect_stack_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::EffectStack)
}

pub(super) fn final_composite_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::FinalComposite)
}

pub(super) fn fxaa_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Fxaa)
}

fn product_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
    kind: PostProcessEffectKind,
) -> Result<(), String> {
    let gpu = context.require_gpu()?;
    let required_resources = {
        let frame_extract = gpu.frame_extract();
        let Some(node) = frame_extract
            .post_process
            .graph
            .nodes
            .iter()
            .find(|node| node.kind == kind)
        else {
            return Ok(());
        };
        node.required_inputs
            .iter()
            .chain(&node.produced_outputs)
            .cloned()
            .collect::<Vec<_>>()
    };

    for resource in required_resources {
        gpu.resources.require_texture_view(&resource)?;
    }

    Ok(())
}

pub(super) fn ssao_executor(context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_ssao_to_resources(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
    )
}

pub(super) fn clustered_lighting_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_clustered_lighting_to_resources(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::LIGHT_LIST,
    )
}

pub(super) fn bloom_extract_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_bloom_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::BLOOM,
    )
}

pub(super) fn post_stack_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_post_process_stack(&pass_name)
}
