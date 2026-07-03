use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::super::paint_theme::METRICS;
use super::super::super::render_commands::HostPaintCommand;
use super::super::style::table_cell_color;
use super::allocation::{table_column_alignment, TableColumnAlignment};
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
        let cell_rect = table_cell_rect(node, rect, index);
        if cell_rect.width <= 0.0 {
            continue;
        }
        commands.push(text_command(
            text_frame_for_cell(cell_rect, cell, index),
            clip,
            order,
            cell,
            table_cell_color(node, index),
            opacity,
        ));
    }
}

fn text_frame_for_cell(rect: FrameRect, text: &str, index: usize) -> FrameRect {
    match table_column_alignment(index) {
        TableColumnAlignment::Left => rect,
        TableColumnAlignment::Right => right_aligned_text_frame(rect, text),
    }
}

fn right_aligned_text_frame(rect: FrameRect, text: &str) -> FrameRect {
    let measured_width =
        measure_runtime_text_width(text, TABLE_CELL_FONT_SIZE) + METRICS.text_clip_guard;
    let width = measured_width.min(rect.width).max(0.0);
    FrameRect {
        x: rect.x + (rect.width - width).max(0.0),
        width,
        ..rect
    }
}

fn text_command(
    rect: FrameRect,
    _clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) -> HostPaintCommand {
    let clip = rect.clone();
    HostPaintCommand::text(
        rect,
        Some(clip),
        order,
        text.to_string(),
        color,
        TABLE_CELL_FONT_SIZE,
        TABLE_CELL_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    )
}
