//! 轴标签优先显示节点声明文本，空文本才回退到 ID 所代表的 X/Y/Z；创建单行文字命令后抑制普通标签回退。
//! text rect 与祖先 clip 必须对应当前节点槽。

mod geometry;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::metrics::axis_label_metrics;

use geometry::axis_label_text_rect;
use style::axis_label_text_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    axis: &str,
    opacity: f32,
) {
    let label = axis_label_text(node, axis);
    let metrics = axis_label_metrics();
    let style = axis_label_text_command_style(node, &metrics);
    let text_rect = axis_label_text_rect(rect, &metrics);
    if !text_rect.x.is_finite()
        || !text_rect.y.is_finite()
        || !text_rect.width.is_finite()
        || !text_rect.height.is_finite()
        || text_rect.width <= f32::EPSILON
        || text_rect.height <= f32::EPSILON
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label.to_string(),
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}

fn axis_label_text<'a>(node: &'a TemplatePaneNodeData, axis: &'a str) -> &'a str {
    let text = node.text.trim();
    if text.is_empty() {
        axis
    } else {
        text
    }
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
