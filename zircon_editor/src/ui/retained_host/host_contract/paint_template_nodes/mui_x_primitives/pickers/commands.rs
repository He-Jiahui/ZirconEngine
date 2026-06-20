use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::field::push_picker_field;
use super::popup::push_picker_popup_preview;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_date_time_picker(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let field = push_picker_field(commands, node, rect, clip, order, opacity);
    push_picker_popup_preview(commands, node, rect, &field, clip, order + 3, opacity);
}
