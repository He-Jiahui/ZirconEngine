use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::{data_grid_header_height, data_grid_row_height};
use super::rows::push_data_grid_rows;
use super::surface::{push_data_grid_header, push_data_grid_surface};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_data_grid(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::node_radius(node).max(4.0);
    push_data_grid_surface(commands, node, rect, clip, order, radius, opacity);

    let header_height = data_grid_header_height(rect);
    push_data_grid_header(
        commands,
        rect,
        clip,
        order + 1,
        radius,
        header_height,
        opacity,
    );

    push_data_grid_rows(
        commands,
        node,
        rect,
        clip,
        order,
        rect.y + header_height,
        data_grid_row_height(rect),
        opacity,
    );
}
