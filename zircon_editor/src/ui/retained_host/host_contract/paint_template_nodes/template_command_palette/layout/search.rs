use super::super::super::super::data::FrameRect;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel_rect.x + metrics.panel_padding_x,
        y: panel_rect.y + metrics.search_top,
        width: (panel_rect.width - metrics.panel_padding_x * 2.0).max(1.0),
        height: metrics.search_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_text_rect(
    search_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: search_rect.x + metrics.search_text_x,
        y: search_rect.y + metrics.search_text_y,
        width: (search_rect.width - metrics.search_text_x * 2.0).max(1.0),
        height: metrics.line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_icon_rect(
    search_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: search_rect.x + metrics.search_icon_x,
        y: search_rect.y + (search_rect.height - metrics.search_icon_size).max(0.0) * 0.5,
        width: metrics.search_icon_size,
        height: metrics.search_icon_size,
    }
}
