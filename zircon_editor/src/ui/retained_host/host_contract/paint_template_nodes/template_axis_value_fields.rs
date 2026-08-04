use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;

mod geometry;
mod identity;
mod layers;
mod metrics;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_axis_value_fields_tests/mod.rs"]
mod tests;

use geometry::axis_field_rect;
use identity::is_workbench_axis_value_field;
use layers::value_text_order;
use surface::push_axis_field_surface;
use text::push_axis_field_value;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_value_field_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_axis_value_field(node) {
        return false;
    }

    let field = axis_field_rect(rect);
    if field.width <= 0.0 || field.height <= 0.0 {
        return true;
    }
    let Some(clip) = intersect(&field, clip) else {
        return true;
    };

    push_axis_field_surface(commands, node, &field, &clip, order, opacity);
    push_axis_field_value(
        commands,
        node,
        &field,
        &clip,
        value_text_order(order),
        opacity,
    );
    true
}
