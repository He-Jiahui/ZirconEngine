use super::super::super::super::data::FrameRect;
use super::common::symmetric_extent;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    let y = panel_rect.y + metrics.empty_text_y;
    let content_bottom = panel_rect.y + panel_rect.height - metrics.panel_padding_x;
    FrameRect {
        x: panel_rect.x + metrics.panel_padding_x,
        y,
        width: (panel_rect.width - symmetric_extent(metrics.panel_padding_x))
            .max(metrics.min_frame_extent),
        height: (content_bottom - y).max(0.0),
    }
}
