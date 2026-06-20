use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{scalar_field_rect, value_text_rect};
use super::super::text::text_command;
use super::surface::push_property_value_field_surface;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scalar_value_commands(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    order: i32,
    value: &str,
    opacity: f32,
) {
    let field_rect = scalar_field_rect(rect);
    push_property_value_field_surface(
        commands,
        &field_rect,
        clip,
        order,
        value_field_border_color(node),
        opacity,
    );
    commands.push(text_command(
        value_text_rect(&field_rect),
        clip,
        order + 1,
        value,
        PALETTE.text,
        opacity,
    ));
}

fn value_field_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.focused || node.selected || node.pressed {
        PALETTE.focus_ring
    } else {
        PALETTE.border
    }
}
