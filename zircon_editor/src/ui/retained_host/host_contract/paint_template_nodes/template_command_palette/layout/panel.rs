use super::super::super::super::data::FrameRect;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel_rect.x + metrics.panel_padding_x,
        y: panel_rect.y + metrics.empty_text_y,
        width: (panel_rect.width - metrics.panel_padding_x * 2.0).max(1.0),
        height: metrics.line_height,
    }
}
