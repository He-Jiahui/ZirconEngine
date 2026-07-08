use super::super::super::super::data::FrameRect;
use super::common::{centered_offset, symmetric_extent};
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_rect(
    panel_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel_rect.x + metrics.panel_padding_x,
        y: panel_rect.y + metrics.search_top,
        width: (panel_rect.width - symmetric_extent(metrics.panel_padding_x))
            .max(metrics.min_frame_extent),
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
        width: (search_rect.width - symmetric_extent(metrics.search_text_x))
            .max(metrics.min_frame_extent),
        height: metrics.line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_icon_rect(
    search_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: search_rect.x + metrics.search_icon_x,
        y: search_rect.y + centered_offset(search_rect.height, metrics.search_icon_size),
        width: metrics.search_icon_size,
        height: metrics.search_icon_size,
    }
}
