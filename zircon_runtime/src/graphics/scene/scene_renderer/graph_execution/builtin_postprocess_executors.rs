use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, RenderExposureMode,
};
use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassExecutionContext;

use self::frame_effects::{
    frame_uses_depth_of_field, frame_uses_reconstructed_motion_vectors, frame_uses_scene_velocity,
    frame_uses_screen_space_reflection, frame_uses_taa,
};
use self::graph_resources::product_postprocess_executor;
use self::resource_routing::{
    bloom_input_resource, output_transfer_input_resource, output_transfer_output_resource,
    terminal_anti_alias_input_resource, upscale_input_resource,
};

mod frame_effects;
mod graph_resources;
mod resource_routing;

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

pub(super) fn taa_reactive_mask_mesh_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    if !frame_uses_taa(context)? {
        return Ok(());
    }
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_taa_reactive_mask_mesh_to_resource(
        &pass_name,
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        PostProcessGraphResourceNames::SCENE_DEPTH,
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
    let input_resource = upscale_input_resource(context);
    let gpu = context.require_gpu()?;
    gpu.record_upscale_to_resource(
        &pass_name,
        input_resource,
        PostProcessGraphResourceNames::UPSCALED,
        attachment_ops,
    )
}

pub(super) fn fxaa_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Fxaa)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::FINAL_COMPOSITED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let input_resource = terminal_anti_alias_input_resource(context);
    let gpu = context.require_gpu()?;
    gpu.record_fxaa_to_resource(
        &pass_name,
        input_resource,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        attachment_ops,
    )
}

pub(super) fn smaa_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    product_postprocess_executor(context, PostProcessEffectKind::Smaa)?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::FINAL_COMPOSITED)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let input_resource = terminal_anti_alias_input_resource(context);
    let gpu = context.require_gpu()?;
    gpu.record_smaa_to_resource(
        &pass_name,
        input_resource,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
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
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_post_process_stack(&pass_name)
}
