//! 字段值优先取 value_text，空时回退 text；文字仅在收紧后的内容区与祖先裁剪有交集时绘制。
//! 这里显示编辑结果而不处理文本输入焦点或事件提交。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_axis_value_field_style::axis_field_text_color;
use super::metrics::{axis_value_field_metrics, AxisValueFieldMetrics};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_field_value(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    field: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let value = axis_field_value(node);
    if value.is_empty() {
        return;
    }

    let metrics = axis_value_field_metrics();
    let text_rect = axis_field_text_rect(field, metrics);
    if text_rect.width <= 0.0 || text_rect.height <= 0.0 {
        return;
    }
    let Some(text_clip) = intersect(&text_rect, clip) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(text_clip),
        order,
        value.to_string(),
        axis_field_text_color(node),
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn axis_field_text_rect(field: &FrameRect, metrics: AxisValueFieldMetrics) -> FrameRect {
    let inset_x = metrics.text_inset_x.min(field.width.max(0.0) * 0.5);
    let line_height = metrics.line_height.min(field.height.max(0.0));
    FrameRect {
        x: field.x + inset_x,
        y: field.y + (field.height - line_height).max(0.0) * 0.5,
        width: (field.width - inset_x * 2.0).max(0.0),
        height: line_height,
    }
}

fn axis_field_value(node: &TemplatePaneNodeData) -> &str {
    let value = node.value_text.trim();
    if value.is_empty() {
        node.text.trim()
    } else {
        value
    }
}

#[cfg(test)]
#[path = "tests/text.rs"]
mod tests;
