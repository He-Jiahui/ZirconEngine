use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::style::{tree_view_marker_color, tree_view_row_color, tree_view_surface_color};

const MUI_X_TREE_ROW_COUNT: i32 = 3;
const MUI_X_TREE_ROW_HORIZONTAL_INSET: f32 = 4.0;
const MUI_X_TREE_ROW_INDENT_STEP: f32 = 6.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_view(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::node_radius(node).max(4.0);
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::node_background(node).unwrap_or_else(|| tree_view_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let row_height = ((rect.height - MUI_X_TREE_ROW_HORIZONTAL_INSET * 2.0)
        / MUI_X_TREE_ROW_COUNT as f32)
        .max(6.0);
    for row in 0..MUI_X_TREE_ROW_COUNT {
        push_tree_view_row(commands, node, rect, clip, order, opacity, row_height, row);
    }
}

fn push_tree_view_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    row_height: f32,
    row: i32,
) {
    let row_y = rect.y + MUI_X_TREE_ROW_HORIZONTAL_INSET + row as f32 * row_height;
    let row_indent = row as f32 * MUI_X_TREE_ROW_INDENT_STEP;
    let row_rect = FrameRect {
        x: rect.x + MUI_X_TREE_ROW_HORIZONTAL_INSET + row_indent,
        y: row_y,
        width: (rect.width - MUI_X_TREE_ROW_HORIZONTAL_INSET * 2.0 - row_indent).max(1.0),
        height: (row_height - 1.0).max(1.0),
    };
    super::super::push_quad(
        commands,
        row_rect.clone(),
        clip,
        order + 1 + row,
        tree_view_row_color(node, row),
        0.0,
        4.0,
        opacity,
    );
    let marker_size = (row_rect.height * 0.45).max(3.0).min(6.0);
    super::super::push_quad(
        commands,
        FrameRect {
            x: row_rect.x + 3.0,
            y: row_rect.y + (row_rect.height - marker_size) * 0.5,
            width: marker_size,
            height: marker_size,
        },
        clip,
        order + 5 + row,
        tree_view_marker_color(node, row),
        0.0,
        marker_size * 0.5,
        opacity,
    );
}
