//! 将内建场景 pass 的逻辑资源读写交给受限 GPU 上下文。
//! 每个入口须由 executor 注册表配对；实际视图、附件操作和可用 renderer 服务由本帧图上下文提供。

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::transparency::{
    HALF_RES_TRANSPARENCY_COMPOSITE_EXECUTOR_ID,
    HALF_RES_TRANSPARENCY_DEPTH_DOWNSAMPLE_EXECUTOR_ID, HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID,
    HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID,
};
use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassExecutionContext;

pub(super) fn transmission_scene_copy_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let step_index =
        crate::graphics::pipeline::transmission_scene_copy_step_index(context.executor_id.as_str())
            .ok_or_else(|| {
                format!(
                    "executor `{}` is not a transmission scene-copy executor",
                    context.executor_id
                )
            })?;
    let gpu = context.require_gpu()?;
    if !gpu.transmission_step_has_commands(step_index)? {
        return Ok(());
    }
    gpu.record_transmission_scene_color_copy(
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
    )
}

pub(super) fn transmission_mesh_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let step_index =
        crate::graphics::pipeline::transmission_mesh_step_index(context.executor_id.as_str())
            .ok_or_else(|| {
                format!(
                    "executor `{}` is not a transmission mesh executor",
                    context.executor_id
                )
            })?;
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let depth_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_DEPTH)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let gpu = context.require_gpu()?;
    gpu.record_transmission_step_to_resources(
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        step_index,
        attachment_ops,
        depth_attachment_ops,
    )
}

pub(super) fn advanced_pbr_opaque_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let gpu = context.require_gpu()?;
    gpu.record_advanced_pbr_opaque_to_resources(
        PostProcessGraphResourceNames::SCENE_COLOR,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        attachment_ops,
        RenderGraphAttachmentOps::load_store(),
    )
}

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
    let (color_resource, depth_resource) =
        if context.executor_id.as_str() == HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID {
            (
                PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
                PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
            )
        } else {
            (
                PostProcessGraphResourceNames::SCENE_COLOR,
                PostProcessGraphResourceNames::SCENE_DEPTH,
            )
        };
    let gpu = context.require_gpu()?;
    gpu.record_particle_billboards_to_resources(color_resource, depth_resource)
}

pub(super) fn half_resolution_transparency_depth_downsample_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let color_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let depth_attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH)
        .unwrap_or_else(RenderGraphAttachmentOps::clear_store);
    let gpu = context.require_gpu()?;
    gpu.record_half_resolution_transparency_depth_downsample(
        color_attachment_ops,
        depth_attachment_ops,
    )
}

pub(super) fn half_resolution_transparency_composite_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let attachment_ops = context
        .attachment_ops_for_write(PostProcessGraphResourceNames::SCENE_COLOR)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let gpu = context.require_gpu()?;
    gpu.record_half_resolution_transparency_composite(attachment_ops)
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
    let (color_resource, depth_resource) =
        if context.executor_id.as_str() == HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID {
            (
                PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
                PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
            )
        } else {
            (
                PostProcessGraphResourceNames::SCENE_COLOR,
                PostProcessGraphResourceNames::SCENE_DEPTH,
            )
        };
    let attachment_ops = context
        .attachment_ops_for_write(color_resource)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let depth_attachment_ops = context
        .attachment_ops_for_write(depth_resource)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let is_half_resolution = context.executor_id.as_str() == HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID;
    let gpu = context.require_gpu()?;
    if is_half_resolution {
        gpu.record_half_resolution_transparent_mesh_to_resources(
            color_resource,
            depth_resource,
            attachment_ops,
            depth_attachment_ops,
        )
    } else {
        gpu.record_mesh_stage_to_resources(
            color_resource,
            depth_resource,
            stage,
            attachment_ops,
            depth_attachment_ops,
        )
    }
}

fn mesh_stage_for_executor(executor_id: &str) -> Result<RenderPassStage, String> {
    match executor_id {
        "mesh.opaque" => Ok(RenderPassStage::Opaque3d),
        "mesh.alpha-mask" => Ok(RenderPassStage::AlphaMask3d),
        "mesh.transparent" => Ok(RenderPassStage::Transparent3d),
        HALF_RES_TRANSPARENCY_MESH_EXECUTOR_ID => Ok(RenderPassStage::Transparent3d),
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

// 输出可能经过 UI 合成，也可能直接来自最终颜色；以编译 pass 的读边为准，
// 不能仅凭全局资源表中存在某个视图来决定展示哪张图。
pub(super) fn surface_present_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let source_resource_name = if context.declares_resource_name_access(
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        crate::render_graph::RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COLOR,
        crate::render_graph::RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COLOR
    } else {
        return Err(
            "surface-present graph pass is missing its final output read dependency".to_string(),
        );
    };
    context
        .require_gpu()?
        .record_surface_present(source_resource_name)
}

pub(super) fn output_target_writeback_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let source_resource_name = if context.declares_resource_name_access(
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        crate::render_graph::RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COLOR,
        crate::render_graph::RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COLOR
    } else {
        return Err(
            "output-target-writeback graph pass is missing its final output read dependency"
                .to_string(),
        );
    };
    if !context.declares_resource_name_access(
        crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
        crate::render_graph::RenderGraphResourceAccessKind::Write,
    ) {
        return Err(
            "output-target-writeback graph pass is missing its output target write dependency"
                .to_string(),
        );
    }
    context.require_gpu()?.record_output_target_writeback(
        source_resource_name,
        crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
    )
}

// 直导入路径已经由目标资源绑定承载物理输出；它只记录计划结果，
// 不得再声明写回边或重复编码一次目标拷贝。
pub(super) fn output_target_direct_import_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let source_resource_name = if context.declares_resource_name_access(
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        crate::render_graph::RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COLOR,
        crate::render_graph::RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COLOR
    } else {
        return Err(
            "output-target-direct-import graph pass is missing its final output read dependency"
                .to_string(),
        );
    };
    if context.declares_resource_name_access(
        crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
        crate::render_graph::RenderGraphResourceAccessKind::Write,
    ) {
        return Err(
            "output-target-direct-import graph pass must not declare a writeback destination"
                .to_string(),
        );
    }
    context
        .require_gpu()?
        .record_output_target_direct_import(source_resource_name)
}

pub(super) fn overlay_depth_reconstruct_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    context.require_gpu()?.record_overlay_depth_reconstruction(
        &pass_name,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::VIEWPORT_OVERLAY_DEPTH,
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
        PostProcessGraphResourceNames::VIEWPORT_OVERLAY_DEPTH,
    )
}
