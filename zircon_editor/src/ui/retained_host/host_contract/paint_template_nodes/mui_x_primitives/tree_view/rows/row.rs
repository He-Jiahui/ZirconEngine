use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{tree_view_marker_color, tree_view_row_color};
use super::metrics::{MUI_X_TREE_ROW_HORIZONTAL_INSET, MUI_X_TREE_ROW_INDENT_STEP};

pub(super) fn push_tree_view_row(
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
    super::super::super::push_quad(
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
    super::super::super::push_quad(
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
