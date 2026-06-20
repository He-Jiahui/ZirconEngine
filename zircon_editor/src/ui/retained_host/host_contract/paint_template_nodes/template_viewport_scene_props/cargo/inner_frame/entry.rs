use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::edges::push_cargo_inner_edges;
use super::grid::push_cargo_inner_grid;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_cargo_inner_frame(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_cargo_inner_edges(commands, rect, clip, order, opacity);
    push_cargo_inner_grid(commands, rect, clip, order, opacity);
}
