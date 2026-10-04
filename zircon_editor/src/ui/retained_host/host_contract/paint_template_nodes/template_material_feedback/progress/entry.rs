//! 只选择进度表现形态，保留同一份节点、裁剪和外部透明度交给线形或环形专用绘制器。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::circular_progress::push_circular_progress_command;
use super::super::state::progress_is_circular;
use super::linear::push_linear_progress_commands;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if progress_is_circular(node) {
        push_circular_progress_command(commands, node, rect, clip, order, opacity);
    } else {
        push_linear_progress_commands(commands, node, rect, clip, order, opacity);
    }
}
