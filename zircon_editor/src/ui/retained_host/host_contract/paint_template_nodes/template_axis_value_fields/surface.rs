use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_axis_value_field_style::{
    axis_field_background, axis_field_border, axis_field_border_width,
};

const AXIS_FIELD_RADIUS: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_field_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    field: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        field.clone(),
        Some(clip.clone()),
        order,
        Some(axis_field_background(node)),
        Some(axis_field_border(node)),
        axis_field_border_width(node),
        AXIS_FIELD_RADIUS,
        opacity,
    ));
}
