use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassExecutionContext;

pub(super) fn preview_sky_scene_color_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    record_preview_sky(context, PostProcessGraphResourceNames::SCENE_COLOR)
}

fn record_preview_sky(
    context: &mut RenderPassExecutionContext<'_>,
    color_resource_name: &str,
) -> Result<(), String> {
    let color_attachment_ops = context
        .attachment_ops_for_write(color_resource_name)
        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
    let depth_attachment_ops = RenderGraphAttachmentOps::load_store();
    let pass_name = context.pass_name.clone();
    let gpu = context.require_gpu()?;
    gpu.record_preview_sky_to_resources(
        &pass_name,
        color_resource_name,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        color_attachment_ops,
        depth_attachment_ops,
    )
}
