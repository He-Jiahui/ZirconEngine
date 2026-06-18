use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{is_table_selected, is_table_tail};
use super::style::{
    table_row_background, table_row_border, table_row_border_width, table_row_style,
};

const TABLE_ROW_RADIUS: f32 = 3.0;

pub(super) fn push_table_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(table_row_background(node)),
        table_row_border(node),
        table_row_border_width(node),
        TABLE_ROW_RADIUS,
        opacity,
    ));

    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - 1.0).max(0.0),
            width: rect.width,
            height: 1.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(table_row_style(node).separator),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(super) fn table_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    if is_table_tail(node) || is_table_selected(node) {
        FrameRect {
            x: rect.x + node.layout_offset_x,
            y: rect.y + node.layout_offset_y,
            width: rect.width,
            height: rect.height,
        }
    } else {
        rect.clone()
    }
}
