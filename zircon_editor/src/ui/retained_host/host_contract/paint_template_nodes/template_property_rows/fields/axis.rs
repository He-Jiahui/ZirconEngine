use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_property_axis_values::PropertyAxisValue;
use super::super::super::template_row_metrics::workbench_row_palette;
use super::super::layers::field_text_order;
use super::super::layout::{axis_field_rect, axis_label_rect, value_text_rect};
use super::super::text::text_command;
use super::surface::push_property_value_field_surface;
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_value_commands(
    commands: &mut Vec<HostPaintCommand>,
    axis_values: &[PropertyAxisValue],
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let count = axis_values.len().min(4);
    let palette = workbench_row_palette();

    for (index, axis_value) in axis_values.iter().take(count).enumerate() {
        commands.push(text_command(
            axis_label_rect(rect, count, index),
            clip,
            order,
            axis_value.axis.as_str(),
            palette.property_axis_label_text,
            opacity,
        ));

        let field_rect = axis_field_rect(rect, count, index);
        push_property_value_field_surface(
            commands,
            &field_rect,
            clip,
            order,
            palette.property_field_border,
            opacity,
        );
        commands.push(text_command(
            value_text_rect(&field_rect),
            clip,
            field_text_order(order),
            axis_value.value.as_str(),
            palette.property_value_text,
            opacity,
        ));
    }
}
