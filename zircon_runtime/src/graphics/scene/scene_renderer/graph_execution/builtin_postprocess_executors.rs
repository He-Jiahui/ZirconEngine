use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, RenderPostProcessEffectStackSettings,
};
use crate::render_graph::RenderGraphAttachmentOps;

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
    let required_resources = {
        let gpu = context.require_gpu()?;
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
        if let Some(declaration) = context.resource_resolver().and_then(|resolver| {
            resolver.pass_resource_declaration_by_name(
                &resource,
                crate::render_graph::RenderGraphResourceAccessKind::Read,
            )
        }) {
            context
                .require_gpu()?
                .resources
                .require_texture_view_for_declaration(declaration)?;
        } else {
            context
                .require_gpu()?
                .resources
                .require_texture_view(&resource)?;
        }
    }

    Ok(())
}

fn frame_post_process_effect_stack(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<RenderPostProcessEffectStackSettings, String> {
    Ok(context
        .require_gpu()?
        .frame_extract()
        .post_process
        .effect_stack)
}

fn frame_uses_reconstructed_motion_vectors(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<bool, String> {
    let effect_stack = frame_post_process_effect_stack(context)?;
    Ok(effect_stack.motion_blur.is_enabled() || effect_stack.screen_space_reflection.is_enabled())
}

fn frame_uses_depth_of_field(context: &mut RenderPassExecutionContext<'_>) -> Result<bool, String> {
    Ok(frame_post_process_effect_stack(context)?
        .depth_of_field
        .is_enabled())
}

fn frame_uses_screen_space_reflection(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<bool, String> {
    Ok(frame_post_process_effect_stack(context)?
        .screen_space_reflection
        .is_enabled())
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
        PostProcessGraphResourceNames::HZB_FURTHEST,
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
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        PostProcessGraphResourceNames::LIGHT_LIST,
    )
}

pub(super) fn hzb_build_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_hzb_build_to_resource(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::HZB_FURTHEST,
    )
}

pub(super) fn hzb_occlusion_cull_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_hzb_occlusion_cull_to_indirect_args(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
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

pub(super) fn motion_vector_clear_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_reconstructed_motion_vectors(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_motion_vector_clear_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
        RenderGraphAttachmentOps::clear_store(),
    )
}

pub(super) fn motion_vector_camera_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_reconstructed_motion_vectors(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_motion_vector_camera_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
        RenderGraphAttachmentOps::load_store(),
    )
}

pub(super) fn motion_vector_mesh_object_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_reconstructed_motion_vectors(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_MOTION_VECTOR)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_mesh_motion_vectors_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        attachment_ops,
    )
}

pub(super) fn motion_vector_tile_max_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_reconstructed_motion_vectors(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_motion_vector_tile_max_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        RenderGraphAttachmentOps::clear_store(),
    )
}

pub(super) fn motion_vector_tile_max_coarse_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_reconstructed_motion_vectors(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_motion_vector_tile_max_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        RenderGraphAttachmentOps::clear_store(),
    )
}

pub(super) fn motion_vector_neighbor_max_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_reconstructed_motion_vectors(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_motion_vector_neighbor_max_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        RenderGraphAttachmentOps::clear_store(),
    )
}

pub(super) fn depth_of_field_prepare_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_depth_of_field(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_depth_of_field_prepare_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
    )
}

pub(super) fn screen_space_reflection_resolve_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_screen_space_reflection(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_screen_space_reflection_resolve_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        PostProcessGraphResourceNames::HZB_FURTHEST,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        attachment_ops,
    )
}

pub(super) fn screen_space_reflection_reflection_pyramid_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_screen_space_reflection(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        )
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_screen_space_reflection_reflection_pyramid_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        attachment_ops,
    )
}

pub(super) fn screen_space_reflection_reflection_pyramid_coarse_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_screen_space_reflection(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        )
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_screen_space_reflection_reflection_pyramid_coarse_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        attachment_ops,
    )
}

pub(super) fn screen_space_reflection_specular_occlusion_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_screen_space_reflection(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        )
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_screen_space_reflection_specular_occlusion_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        attachment_ops,
    )
}

pub(super) fn post_stack_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_post_process_stack(&pass_name)
}
