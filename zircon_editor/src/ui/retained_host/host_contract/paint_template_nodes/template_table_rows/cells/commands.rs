use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::style::table_cell_color;
use super::geometry::table_cell_rect;
use super::metrics::{TABLE_CELL_FONT_SIZE, TABLE_COLUMN_RATIOS};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_cells(
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
