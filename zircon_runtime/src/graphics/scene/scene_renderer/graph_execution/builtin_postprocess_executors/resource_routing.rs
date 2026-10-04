//! 根据当前 pass 已声明的读边选择后处理输入。
//! 缺省名称只代表无可选效果时的基线路径，后续资源解析仍会校验声明和绑定。

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::RenderPassExecutionContext;

pub(super) fn output_transfer_output_resource(
    _context: &RenderPassExecutionContext<'_>,
) -> &'static str {
    PostProcessGraphResourceNames::FINAL_COLOR
}

pub(super) fn output_transfer_input_resource(
    context: &RenderPassExecutionContext<'_>,
) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::SECONDARY_UPSCALED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::SECONDARY_UPSCALED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::PRIMARY_UPSCALED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::PRIMARY_UPSCALED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    } else {
        PostProcessGraphResourceNames::TONEMAPPED
    }
}

pub(super) fn upscale_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::PRIMARY_UPSCALED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::PRIMARY_UPSCALED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    } else {
        PostProcessGraphResourceNames::TONEMAPPED
    }
}

pub(super) fn terminal_anti_alias_input_resource(
    context: &RenderPassExecutionContext<'_>,
) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TONEMAPPED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TONEMAPPED
    } else {
        PostProcessGraphResourceNames::FINAL_COMPOSITED
    }
}

pub(super) fn taa_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

pub(super) fn bloom_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
    if context.declares_resource_name_access(
        PostProcessGraphResourceNames::MOTION_BLURRED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

#[cfg(test)]
pub(super) fn uber_input_resource(context: &RenderPassExecutionContext<'_>) -> &'static str {
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
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else if context.declares_resource_name_access(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    ) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

#[cfg(test)]
#[path = "tests/resource_routing.rs"]
mod tests;
