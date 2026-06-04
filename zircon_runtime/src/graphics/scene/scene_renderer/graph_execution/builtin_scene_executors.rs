use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassExecutionContext;

const SHADOW_MAP_RESOURCE: &str = "shadow-map";

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

pub(super) fn depth_prepass_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let normal_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let depth_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_DEPTH)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_depth_prepass_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        normal_attachment_ops,
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
    let material_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_deferred_gbuffer_to_resources(
        &pass_name,
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        attachment_ops,
        material_attachment_ops,
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
        PostProcessGraphResourceNames::FINAL_COLOR,
        PostProcessGraphResourceNames::SCENE_COLOR,
        attachment_ops,
    )
}

pub(super) fn shadow_map_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let attachment_ops = context
        .attachment_ops_for_write(SHADOW_MAP_RESOURCE)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_shadow_map_to_resource(&pass_name, SHADOW_MAP_RESOURCE, attachment_ops)
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
