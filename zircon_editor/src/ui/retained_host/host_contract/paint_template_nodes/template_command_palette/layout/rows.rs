use super::super::super::super::data::FrameRect;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_rect(
    panel_rect: &FrameRect,
    row: usize,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel_rect.x + metrics.row_inset_x,
        y: panel_rect.y + metrics.list_top + row as f32 * metrics.row_height,
        width: (panel_rect.width - metrics.row_inset_x * 2.0).max(1.0),
        height: metrics.row_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_label_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: row_rect.x + metrics.row_text_x,
        y: row_rect.y + metrics.row_text_y,
        width: (row_rect.width - metrics.row_text_x * 2.0).max(1.0),
        height: (row_rect.height - metrics.row_text_y * 2.0).max(metrics.line_height),
    }
}
