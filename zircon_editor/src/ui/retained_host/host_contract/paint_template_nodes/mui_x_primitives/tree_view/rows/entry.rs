use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::tree_view_surface_color;
use super::metrics::{tree_view_row_metrics, MUI_X_TREE_ROW_COUNT};
use super::row::push_tree_view_row;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_view(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = tree_view_row_metrics();
    let radius = super::super::super::node_radius(node).max(metrics.row_radius);
    super::super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::super::node_background(node).unwrap_or_else(|| tree_view_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let row_height =
        ((rect.height - metrics.horizontal_inset * 2.0) / MUI_X_TREE_ROW_COUNT as f32).max(0.0);
    for row in 0..MUI_X_TREE_ROW_COUNT {
        push_tree_view_row(
            commands, node, rect, clip, order, opacity, metrics, row_height, row,
        );
    }
}
