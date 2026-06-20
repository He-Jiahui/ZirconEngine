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
    commands.push(text_command(
        label_text_rect(rect, label_width),
        clip,
        order,
        label,
        text_color(node),
        opacity,
    ));
}
