use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{is_table_header, is_table_tail};
use super::style::table_cell_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TABLE_CELL_FONT_SIZE: f32 = 11.0;
const TABLE_CELL_INSET_X: f32 = 9.0;
const TABLE_CELL_INSET_Y: f32 = 4.0;
pub(super) const TABLE_ACTION_WIDTH: f32 = 24.0;
const TABLE_COLUMN_RATIOS: [f32; 4] = [0.36, 0.27, 0.19, 0.18];

pub(super) fn table_cells(node: &TemplatePaneNodeData) -> Vec<String> {
    let option_cells = (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|cell| cell.to_string())
        .filter(|cell| !cell.trim().is_empty())
        .collect::<Vec<_>>();
    if !option_cells.is_empty() {
        return option_cells;
    }
    split_legacy_table_text(node.text.as_str())
}

pub(super) fn split_legacy_table_text(text: &str) -> Vec<String> {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        [] => Vec::new(),
        [name, kind, size, size_unit, modified_value, modified_unit, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            format!("{size} {size_unit}"),
            format!("{modified_value} {modified_unit}"),
        ],
        [name, kind, size, modified, ..] => vec![
            (*name).to_string(),
            (*kind).to_string(),
            (*size).to_string(),
            (*modified).to_string(),
        ],
        _ => vec![text.trim().to_string()],
    }
}

pub(super) fn push_table_cells(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    cells: &[String],
) {
    for (index, cell) in cells.iter().take(TABLE_COLUMN_RATIOS.len()).enumerate() {
        commands.push(text_command(
            table_cell_rect(node, rect, index),
            clip,
            order,
            cell,
            table_cell_color(node, index),
            opacity,
        ));
    }
}

pub(super) fn table_cell_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let mut x = rect.x + TABLE_CELL_INSET_X + content_offset_x;
    let available_width = (rect.width - TABLE_CELL_INSET_X * 2.0 - TABLE_ACTION_WIDTH).max(1.0);
    for ratio in TABLE_COLUMN_RATIOS.iter().take(index) {
        x += available_width * ratio;
    }
    let width = TABLE_COLUMN_RATIOS
        .get(index)
        .map(|ratio| available_width * ratio)
        .unwrap_or(available_width)
        .max(1.0);
    FrameRect {
        x: x + table_cell_offset_x(node, index),
        y: rect.y + TABLE_CELL_INSET_Y + content_offset_y,
        width: width.max(1.0),
        height: (rect.height - TABLE_CELL_INSET_Y * 2.0).max(1.0),
    }
}

pub(super) fn table_content_offset(node: &TemplatePaneNodeData) -> (f32, f32) {
    if is_table_header(node) || is_table_tail(node) {
        (node.layout_content_offset_x, node.layout_content_offset_y)
    } else {
        (0.0, 0.0)
    }
}

fn table_cell_offset_x(node: &TemplatePaneNodeData, index: usize) -> f32 {
    match index {
        0 => node.layout_first_cell_offset_x,
        1 => node.layout_second_cell_offset_x,
        2 => node.layout_third_cell_offset_x,
        3 => node.layout_fourth_cell_offset_x,
        _ => 0.0,
    }
}

fn text_command(
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) -> HostPaintCommand {
    HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        TABLE_CELL_FONT_SIZE,
        TABLE_CELL_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    )
}
