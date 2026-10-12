use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::super::template_alert_glyphs::push_close_mark;
use super::super::geometry::chip_delete_icon_frame;
use super::super::style::chip_delete_icon_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_delete_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = chip_delete_icon_frame(node, rect);
    let color = chip_delete_icon_color(node);
    push_close_mark(commands, &frame, clip, order, color, opacity);
}
