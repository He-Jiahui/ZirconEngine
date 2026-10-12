//! 标签采用通用节点文字色，值区另取属性字段颜色；两区保持各自裁剪，防止长标签遮挡值。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::text_color;
use super::layout::label_text_rect;
use super::text::text_command;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_property_label_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    label_width: f32,
    opacity: f32,
) {
    let Some(command) = text_command(
        label_text_rect(rect, label_width),
        clip,
        order,
        label,
        text_color(node),
        opacity,
    ) else {
        return;
    };
    commands.push(command);
}
