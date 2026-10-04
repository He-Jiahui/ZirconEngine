//! 拖拽预览标题消费借用的投影内容，优先展示 payload 标签，再退回节点文本、引用和 value。
//! 仅在内容区域有效时复制为命令拥有的字符串，避免不可见反馈引入文本分配。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{layout, style};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_preview_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    preview_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: style::DragOverlayPalette,
    metrics: &layout::DragOverlayMetrics,
) {
    let Some(label) = preview_label(node) else {
        return;
    };
    let text_rect = layout::preview_text_frame(preview_rect, metrics);
    if text_rect.width <= 0.0 || text_rect.height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label.to_string(),
        palette.preview_text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn preview_label(node: &TemplatePaneNodeData) -> Option<&str> {
    [
        node.drag_payload_label.as_str(),
        node.text.as_str(),
        node.drag_payload_reference.as_str(),
        node.value_text.as_str(),
    ]
    .into_iter()
    .map(str::trim)
    .find(|value| !value.is_empty())
}

#[cfg(test)]
#[path = "tests/text_optimization_batch_gu_editor576_tests.rs"]
mod optimization_batch_gu_editor576_tests;
