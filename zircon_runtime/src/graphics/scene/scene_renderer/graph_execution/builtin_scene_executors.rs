use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassExecutionContext;

pub(super) fn sprite_executor(context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    let stage = sprite_stage_for_executor(context.executor_id.as_str())?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let depth_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_DEPTH)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let gpu = context.require_gpu()?;
    gpu.record_sprite_stage_to_resources(
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        stage,
        attachment_ops,
        depth_attachment_ops,
    )
}

fn sprite_stage_for_executor(executor_id: &str) -> Result<RenderPassStage, String> {
    match executor_id {
        "sprite.opaque" => Ok(RenderPassStage::Opaque2d),
        "sprite.alpha-mask" => Ok(RenderPassStage::AlphaMask2d),
        "sprite.transparent" => Ok(RenderPassStage::Transparent2d),
        other => Err(format!("executor `{other}` is not a sprite graph executor")),
    }
}

pub(super) fn particle_billboard_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let gpu = context.require_gpu()?;
    gpu.record_particle_billboards_to_resources(
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    )
}

pub(super) fn depth_prepass_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let depth_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_DEPTH)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_depth_prepass_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        depth_attachment_ops,
    )
}

pub(super) fn mesh_executor(context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    let stage = mesh_stage_for_executor(context.executor_id.as_str())?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let depth_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_DEPTH)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let gpu = context.require_gpu()?;
    gpu.record_mesh_stage_to_resources(
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        stage,
        attachment_ops,
        depth_attachment_ops,
    )
}

fn mesh_stage_for_executor(executor_id: &str) -> Result<RenderPassStage, String> {
    match executor_id {
        "mesh.opaque" => Ok(RenderPassStage::Opaque3d),
        "mesh.alpha-mask" => Ok(RenderPassStage::AlphaMask3d),
        "mesh.transparent" => Ok(RenderPassStage::Transparent3d),
        other => Err(format!("executor `{other}` is not a mesh graph executor")),
    }
}

pub(super) fn deferred_gbuffer_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let normal_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let material_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let emissive_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::GBUFFER_EMISSIVE)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_deferred_gbuffer_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::GBUFFER_EMISSIVE,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        attachment_ops,
        normal_attachment_ops,
        material_attachment_ops,
        emissive_attachment_ops,
    )
}

pub(super) fn deferred_lighting_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_deferred_lighting_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::GBUFFER_EMISSIVE,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SCENE_COLOR,
        attachment_ops,
    )
}

pub(super) fn shadow_atlas_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let shadow_atlas_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SHADOW_ATLAS)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_shadow_atlas_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::SHADOW_ATLAS,
        shadow_atlas_attachment_ops,
    )
}

pub(super) fn screen_space_ui_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::VIEWPORT_OUTPUT)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let gpu = context.require_gpu()?;
    gpu.record_screen_space_ui_to_resource(
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        attachment_ops,
    )
}

pub(super) fn overlay_gizmo_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_overlay_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    )
}
