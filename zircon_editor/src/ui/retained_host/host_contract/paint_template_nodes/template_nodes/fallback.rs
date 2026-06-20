use super::super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::material_primitives::{
    push_material_primitive_commands, push_material_text_field_surface_commands,
};
use super::super::mui_x_primitives::push_mui_x_primitive_commands;
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_images::push_template_image_command;
use super::super::template_node_surface::push_template_surface_fallback_commands;
use super::super::template_node_text::push_template_text_fallback_command;
use super::super::template_popup_rows::push_template_popup_row_commands;
use super::super::template_property_rows::push_property_row_text_commands;
use super::super::template_table_rows::push_table_row_text_commands;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_fallback_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    node_clip: &FrameRect,
    origin: &FrameRect,
    pane_clip: &FrameRect,
    order: i32,
    opacity: f32,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if push_material_primitive_commands(commands, node, rect, node_clip, order, opacity) {
        return;
    }

    let draws_mui_x_primitive =
        push_mui_x_primitive_commands(commands, node, rect, node_clip, order, opacity);
    let draws_text_field_surface =
        push_material_text_field_surface_commands(commands, node, rect, node_clip, order, opacity);

    push_template_surface_fallback_commands(
        commands,
        node,
        rect,
        node_clip,
        order,
        opacity,
        draws_mui_x_primitive || draws_text_field_surface,
    );

    push_template_image_command(commands, node, rect, node_clip, order + 2, opacity);

    let property_row_text_painted =
        push_property_row_text_commands(commands, node, rect, node_clip, order + 3, opacity);
    let table_row_text_painted = !property_row_text_painted
        && push_table_row_text_commands(commands, node, rect, node_clip, order + 3, opacity);

    push_template_text_fallback_command(
        commands,
        node,
        rect,
        node_clip,
        order + 3,
        text_input_focus,
        property_row_text_painted,
        table_row_text_painted,
        opacity,
    );

    push_template_popup_row_commands(commands, node, rect, origin, pane_clip, order, opacity);
}
