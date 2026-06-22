use crate::core::framework::render::{
    AntiAliasMode, PostProcessEffectKind, PostProcessGraphResourceNames, RenderExposureMode,
    RenderPostProcessEffectStackSettings,
};
use crate::render_graph::{
    RenderGraphAttachmentOps, RenderGraphExternalResourceType, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

use super::RenderPassExecutionContext;

pub(super) fn bloom_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Bloom)
}

pub(super) fn color_lut_bake_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let should_execute = {
        let gpu = context.require_gpu()?;
        gpu.frame_extract()
            .post_process
            .graph
            .nodes
            .iter()
            .any(|node| node.kind == PostProcessEffectKind::ColorLutBake)
    };
    if !should_execute {
        return Ok(());
    }

    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_color_lut_bake_to_resource(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::COLOR_LUT,
    )
}

pub(super) fn taa_resolve_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_taa(context)? {
        return Ok(());
    }
    let taa_output_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::TAA_OUTPUT)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let taa_history_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_taa_resolve_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        PostProcessGraphResourceNames::TAA_OUTPUT,
        PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
        taa_output_attachment_ops,
        taa_history_attachment_ops,
    )
}

pub(super) fn taa_reactive_mask_clear_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_taa(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::TAA_REACTIVE_MASK)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_taa_reactive_mask_clear_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        attachment_ops,
    )
}

pub(super) fn taa_reactive_mask_mesh_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_taa(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::TAA_REACTIVE_MASK)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_taa_reactive_mask_mesh_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        attachment_ops,
    )
}

pub(super) fn output_transfer_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let input_resource = output_transfer_input_resource(context);
    let output_resource = output_transfer_output_resource(context);
    let attachment_ops = context
        .attachment_ops_for_write(output_resource)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_output_transfer_to_resource(
        &pass_name,
        input_resource,
        output_resource,
        attachment_ops,
    )
}

pub(super) fn upscale_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Upscale)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::UPSCALED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_upscale_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::TONEMAPPED,
        PostProcessGraphResourceNames::UPSCALED,
        attachment_ops,
    )
}

pub(super) fn fxaa_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Fxaa)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::FINAL_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_fxaa_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        PostProcessGraphResourceNames::FINAL_COLOR,
        attachment_ops,
    )
}

pub(super) fn smaa_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Smaa)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::FINAL_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_smaa_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        PostProcessGraphResourceNames::FINAL_COLOR,
        attachment_ops,
    )
}

pub(super) fn motion_blur_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::MotionBlur)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::MOTION_BLURRED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_motion_blur_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::MOTION_BLURRED,
        attachment_ops,
    )
}

pub(super) fn blur_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Blur)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::BLURRED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_blur_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::BLURRED,
        attachment_ops,
    )
}

pub(super) fn depth_of_field_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::DepthOfField)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::DEPTH_OF_FIELDED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_depth_of_field_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        attachment_ops,
    )
}

pub(super) fn scene_composite_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::SceneComposite)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COMPOSITED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_scene_composite_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        attachment_ops,
    )
}

pub(super) fn exposure_histogram_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let exposure_mode = context
        .require_gpu()?
        .frame_extract()
        .post_process
        .exposure
        .mode;
    if exposure_mode != RenderExposureMode::Histogram {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_exposure_histogram_to_resource(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM,
    )
}

pub(super) fn exposure_resolve_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    gpu.record_exposure_resolve_to_resource(
        &pass_name,
        &executor_id,
        PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM,
        PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
        PostProcessGraphResourceNames::EXPOSURE_CURRENT,
    )
}

fn product_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
    kind: PostProcessEffectKind,
) -> Result<(), String> {
    let (required_inputs, produced_outputs) = {
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
        (node.required_inputs.clone(), node.produced_outputs.clone())
    };

    for resource in required_inputs {
        require_graph_resource_by_name(context, &resource, RenderGraphResourceAccessKind::Read)?;
    }
    for resource in produced_outputs {
        require_graph_resource_by_name(context, &resource, RenderGraphResourceAccessKind::Write)?;
    }

    Ok(())
}

fn require_graph_resource_by_name(
    context: &mut RenderPassExecutionContext<'_>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Result<(), String> {
    let Some(kind) = pass_resource_kind(context, resource_name, access) else {
        return Err(format!(
            "render graph pass `{}` did not declare {:?} access for resource `{resource_name}`",
            context.pass_name, access
        ));
    };
    match kind {
        RenderGraphResourceKind::TransientTexture => context
            .require_texture_view_by_name(resource_name, access)
            .map(|_| ()),
        RenderGraphResourceKind::TransientBuffer => context
            .require_buffer_by_name(resource_name, access)
            .map(|_| ()),
        RenderGraphResourceKind::External => match external_resource_type(context, resource_name) {
            RenderGraphExternalResourceType::Buffer => context
                .require_buffer_by_name(resource_name, access)
                .map(|_| ()),
            RenderGraphExternalResourceType::Texture | RenderGraphExternalResourceType::Unknown => {
                context
                    .require_texture_view_by_name(resource_name, access)
                    .map(|_| ())
            }
        },
    }
}

fn pass_resource_kind(
    context: &RenderPassExecutionContext<'_>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Option<RenderGraphResourceKind> {
    if let Some(resolver) = context.resource_resolver() {
        return resolver
            .pass_resource_access_by_name(resource_name, access)
            .map(|resource| resource.kind);
    }
    context
        .resources
        .iter()
        .find(|resource| resource.name == resource_name && resource.access == access)
        .map(|resource| resource.kind)
}

fn external_resource_type(
    context: &RenderPassExecutionContext<'_>,
    resource_name: &str,
) -> RenderGraphExternalResourceType {
    context
        .resource_resolver()
        .and_then(|resolver| resolver.resource_declaration_by_name(resource_name))
        .map(|declaration| declaration.external_binding.resource_type)
        .unwrap_or(RenderGraphExternalResourceType::Unknown)
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

fn output_transfer_output_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Write,
    ) || context
        .attachment_ops_for_write(PostProcessGraphResourceNames::FINAL_COMPOSITED)
        .is_some()
    {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    } else {
        PostProcessGraphResourceNames::FINAL_COLOR
    }
}

fn output_transfer_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::UPSCALED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::UPSCALED
    } else {
        PostProcessGraphResourceNames::TONEMAPPED
    }
}

fn bloom_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::MOTION_BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

fn uber_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::SCENE_COMPOSITED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::MOTION_BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

fn frame_uses_scene_velocity(context: &mut RenderPassExecutionContext<'_>) -> Result<bool, String> {
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

fn frame_uses_taa(context: &mut RenderPassExecutionContext<'_>) -> Result<bool, String> {
    Ok(context.require_gpu()?.frame_extract().view.anti_alias.mode == AntiAliasMode::Taa)
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
    let scene_color_resource = bloom_input_resource(context);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_bloom_to_resources(
        &pass_name,
        scene_color_resource,
        PostProcessGraphResourceNames::BLOOM,
    )
}

pub(super) fn velocity_camera_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_scene_velocity(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_velocity_camera_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        RenderGraphAttachmentOps::load_store(),
    )
}

pub(super) fn velocity_mesh_object_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_scene_velocity(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_VELOCITY)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_velocity_object_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        attachment_ops,
    )
}

pub(super) fn particle_velocity_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_scene_velocity(context)? {
        return Ok(());
    }
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_VELOCITY)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_particle_velocity_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_VELOCITY,
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
        PostProcessGraphResourceNames::SCENE_VELOCITY,
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

pub(super) fn uber_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let _input_resource = uber_input_resource(context);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_post_process_stack(&pass_name)
}

#[cfg(test)]
mod tests {
    use super::{
        bloom_input_resource, output_transfer_input_resource, output_transfer_output_resource,
        uber_input_resource,
    };
    use crate::core::framework::render::PostProcessGraphResourceNames;
    use crate::graphics::RenderPassExecutorId;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
        RenderGraphResourceKind,
    };

    use super::RenderPassExecutionContext;

    #[test]
    fn output_transfer_executor_targets_terminal_input_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            output_transfer_output_resource(&context),
            PostProcessGraphResourceNames::FINAL_COMPOSITED
        );
    }

    #[test]
    fn output_transfer_executor_defaults_to_final_color() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            Vec::new(),
        );

        assert_eq!(
            output_transfer_output_resource(&context),
            PostProcessGraphResourceNames::FINAL_COLOR
        );
    }

    #[test]
    fn output_transfer_executor_reads_upscaled_input_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::UPSCALED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            output_transfer_input_resource(&context),
            PostProcessGraphResourceNames::UPSCALED
        );
    }

    #[test]
    fn output_transfer_executor_defaults_to_tonemapped_input() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "output-transfer",
            RenderPassExecutorId::new("post.output-transfer"),
            QueueLane::Graphics,
            PassFlags::default(),
            Vec::new(),
        );

        assert_eq!(
            output_transfer_input_resource(&context),
            PostProcessGraphResourceNames::TONEMAPPED
        );
    }

    #[test]
    fn bloom_executor_reads_motion_blurred_source_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "bloom-extract",
            RenderPassExecutorId::new("post.bloom-extract"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::MOTION_BLURRED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            bloom_input_resource(&context),
            PostProcessGraphResourceNames::MOTION_BLURRED
        );
    }

    #[test]
    fn bloom_executor_falls_back_to_scene_color_input() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "bloom-extract",
            RenderPassExecutorId::new("post.bloom-extract"),
            QueueLane::Graphics,
            PassFlags::default(),
            Vec::new(),
        );

        assert_eq!(
            bloom_input_resource(&context),
            PostProcessGraphResourceNames::SCENE_COLOR
        );
    }

    #[test]
    fn uber_executor_reads_scene_composited_source_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "uber",
            RenderPassExecutorId::new("post.uber"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            uber_input_resource(&context),
            PostProcessGraphResourceNames::SCENE_COMPOSITED
        );
    }

    #[test]
    fn uber_executor_reads_blurred_source_when_declared() {
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "uber",
            RenderPassExecutorId::new("post.uber"),
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::BLURRED.to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            }],
        );

        assert_eq!(
            uber_input_resource(&context),
            PostProcessGraphResourceNames::BLURRED
        );
    }
}
