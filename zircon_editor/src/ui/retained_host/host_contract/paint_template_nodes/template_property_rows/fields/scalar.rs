use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_row_metrics::{workbench_row_palette, WorkbenchRowPalette};
use super::super::layout::{scalar_field_rect, value_text_rect};
use super::super::text::text_command;
use super::surface::push_property_value_field_surface;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scalar_value_commands(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    order: i32,
    value: &str,
    opacity: f32,
) {
    let palette = workbench_row_palette();
    let field_rect = scalar_field_rect(rect);
    push_property_value_field_surface(
        commands,
        &field_rect,
        clip,
        order,
        value_field_border_color(node, palette),
        opacity,
    );
    commands.push(text_command(
        value_text_rect(&field_rect),
        clip,
        order + 1,
        value,
        palette.property_value_text,
        opacity,
    ));
}

fn value_field_border_color(node: &TemplatePaneNodeData, palette: WorkbenchRowPalette) -> [u8; 4] {
    if node.focused || node.selected || node.pressed {
        palette.property_field_focus_border
    } else {
        palette.property_field_border
    }
}
