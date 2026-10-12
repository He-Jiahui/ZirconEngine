//! 按已分配单元格框生成最多四条文字命令，数值列右对齐且裁剪到自身区域，保持跨行列边界稳定。
//! 单元格声明位置与颜色由 cells/geometry 和共享 row style 负责。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::render_commands::HostPaintCommand;
use super::super::actions::table_action_column_width;
use super::super::style::table_row_style;
use super::allocation::{table_column_alignment, TableColumnAlignment};
use super::geometry::table_cell_rects;
use super::metrics::{table_cell_metrics, WorkbenchTableCellMetrics, TABLE_COLUMN_COUNT};
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
    let metrics = table_cell_metrics();
    if !has_paintable_table_cell_area(rect, metrics) {
        return;
    }
    let cell_count = cells.len().min(TABLE_COLUMN_COUNT);
    commands.reserve(cell_count);
    let cell_rects = table_cell_rects(node, rect);
    let row_style = table_row_style(node);
    for (index, cell) in cells.iter().take(cell_count).enumerate() {
        let cell_rect = cell_rects[index].clone();
        if cell_rect.width <= 0.0 || cell_rect.height <= 0.0 {
            continue;
        }
        let Some(command) = text_command(
            text_frame_for_cell(cell_rect, cell, index, metrics),
            clip,
            order,
            cell,
            row_style.text_for_cell(index),
            metrics,
            opacity,
        ) else {
            continue;
        };
        commands.push(command);
    }
}

fn has_paintable_table_cell_area(rect: &FrameRect, metrics: WorkbenchTableCellMetrics) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width - metrics.inset_x * 2.0 - table_action_column_width() > 0.0
        && rect.height - metrics.inset_y * 2.0 > 0.0
}

fn text_frame_for_cell(
    rect: FrameRect,
    text: &str,
    index: usize,
    metrics: WorkbenchTableCellMetrics,
) -> FrameRect {
    let rect = table_cell_text_frame(rect, metrics);
    match table_column_alignment(index) {
        TableColumnAlignment::Left => rect,
        TableColumnAlignment::Right => right_aligned_text_frame(rect, text, metrics),
    }
}

fn table_cell_text_frame(rect: FrameRect, metrics: WorkbenchTableCellMetrics) -> FrameRect {
    let inset_x = metrics.inset_x.min(rect.width.max(0.0) * 0.5);
    FrameRect {
        x: rect.x + inset_x,
        width: (rect.width - inset_x * 2.0).max(0.0),
        ..rect
    }
}

fn right_aligned_text_frame(
    rect: FrameRect,
    text: &str,
    metrics: WorkbenchTableCellMetrics,
) -> FrameRect {
    let measured_width = measure_runtime_text_width(text, metrics.font_size);
    let width = measured_width.min(rect.width).max(0.0);
    FrameRect {
        x: rect.x + (rect.width - width).max(0.0),
        width,
        ..rect
    }
}

fn text_command(
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    metrics: WorkbenchTableCellMetrics,
    opacity: f32,
) -> Option<HostPaintCommand> {
    if !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return None;
    }
    let clip = intersect(&rect, clip)?;
    Some(HostPaintCommand::text(
        rect,
        Some(clip),
        order,
        text.to_string(),
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ))
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
