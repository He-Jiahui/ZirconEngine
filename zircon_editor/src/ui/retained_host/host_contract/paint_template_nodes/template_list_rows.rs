use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_list_row_glyphs::push_list_row_adornment;

mod identity;
mod style;
mod surface;
mod text;

use identity::is_workbench_list_row;
use style::list_row_adornment_color;
use surface::push_list_row_surface;
use text::push_list_row_label;

#[cfg(test)]
use style::{list_row_background, list_row_style, list_row_text_color};
#[cfg(test)]
#[path = "template_list_rows_tests.rs"]
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

    push_list_row_surface(commands, node, rect, clip, order, opacity);
    push_list_row_label(commands, node, rect, clip, order + 2, opacity);
    push_list_row_adornment(
        commands,
        node,
        rect,
        clip,
        order + 3,
        list_row_adornment_color(node),
        opacity,
    );
    true
}
