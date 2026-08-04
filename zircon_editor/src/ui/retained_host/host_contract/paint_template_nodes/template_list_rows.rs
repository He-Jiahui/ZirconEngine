use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;
use super::template_list_row_glyphs::push_list_row_adornment;

mod geometry;
mod identity;
mod layers;
mod style;
mod surface;
mod text;

use geometry::{has_paintable_list_row_extent, list_row_has_adornment_space};
use identity::is_workbench_list_row;
use layers::{adornment_order, label_order};
use style::list_row_adornment_color;
use surface::push_list_row_surface;
use text::push_list_row_label;

#[cfg(test)]
#[path = "template_list_rows_tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_list_row(node) {
        return false;
    }
    if !has_paintable_list_row_extent(rect) {
        return true;
    }
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };

    push_list_row_surface(commands, node, rect, &clip, order, opacity);
    push_list_row_label(commands, node, rect, &clip, label_order(order), opacity);
    if list_row_has_adornment_space(rect) {
        push_list_row_adornment(
            commands,
            node,
            rect,
            &clip,
            adornment_order(order),
            list_row_adornment_color(node),
            opacity,
        );
    }
    true
}
