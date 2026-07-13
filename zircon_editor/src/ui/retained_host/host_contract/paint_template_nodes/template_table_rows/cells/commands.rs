use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::paint_text::measure_runtime_text_width;
use super::super::super::render_commands::HostPaintCommand;
use super::super::style::table_cell_color;
use super::allocation::{table_column_alignment, TableColumnAlignment};
use super::geometry::table_cell_rect;
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
    for (index, cell) in cells.iter().take(TABLE_COLUMN_COUNT).enumerate() {
        let cell_rect = table_cell_rect(node, rect, index);
        if cell_rect.width <= 0.0 {
            continue;
        }
        let Some(command) = text_command(
            text_frame_for_cell(cell_rect, cell, index, metrics),
            clip,
            order,
            cell,
            table_cell_color(node, index),
            metrics,
            opacity,
        ) else {
            continue;
        };
        commands.push(command);
    }
}

fn text_frame_for_cell(
    rect: FrameRect,
    text: &str,
    index: usize,
    metrics: WorkbenchTableCellMetrics,
) -> FrameRect {
    match table_column_alignment(index) {
        TableColumnAlignment::Left => rect,
        TableColumnAlignment::Right => right_aligned_text_frame(rect, text, metrics),
    }
}

fn right_aligned_text_frame(
    rect: FrameRect,
    text: &str,
    metrics: WorkbenchTableCellMetrics,
) -> FrameRect {
    let measured_width =
        measure_runtime_text_width(text, metrics.font_size) + metrics.text_clip_guard;
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
mod tests {
    use super::*;

    #[test]
    fn table_cell_text_intersects_its_frame_with_the_inherited_clip() {
        let command = text_command(
            frame(10.0, 20.0, 80.0, 18.0),
            &frame(0.0, 24.0, 100.0, 6.0),
            3,
            "Asset.mesh",
            [255, 255, 255, 255],
            table_cell_metrics(),
            1.0,
        )
        .expect("partially visible table text command");

        assert_eq!(command.clip_frame, Some(frame(10.0, 24.0, 80.0, 6.0)));
    }

    #[test]
    fn table_cell_text_outside_the_inherited_clip_emits_no_command() {
        assert!(text_command(
            frame(10.0, 40.0, 80.0, 18.0),
            &frame(0.0, 10.0, 100.0, 20.0),
            3,
            "Asset.mesh",
            [255, 255, 255, 255],
            table_cell_metrics(),
            1.0,
        )
        .is_none());
    }

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
