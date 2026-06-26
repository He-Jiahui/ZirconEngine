use super::super::super::data::FrameRect;
use super::metrics::status_line_height;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_text_rect(
    rect: &FrameRect,
) -> FrameRect {
    let line_height = status_line_height();
    let inset = METRICS.gap_s;
    FrameRect {
        x: rect.x + inset,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - inset * 2.0).max(1.0),
        height: line_height,
    }
}
