use super::super::super::super::data::FrameRect;
use super::common::{centered_offset, symmetric_extent};
use super::metrics::WorkbenchCommandPaletteMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn match_indicator_rect(
    row_rect: &FrameRect,
    metrics: &WorkbenchCommandPaletteMetrics,
) -> FrameRect {
    let height = match_indicator_height(row_rect, metrics);
    FrameRect {
        x: row_rect.x + metrics.match_indicator_left,
        y: row_rect.y + centered_offset(row_rect.height, height),
        width: metrics.match_indicator_width,
        height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn match_indicator_radius(
    metrics: &WorkbenchCommandPaletteMetrics,
) -> f32 {
    metrics.match_indicator_width * 0.5
}

fn match_indicator_height(row_rect: &FrameRect, metrics: &WorkbenchCommandPaletteMetrics) -> f32 {
    metrics.match_indicator_height.min(
        (row_rect.height - symmetric_extent(metrics.match_indicator_width))
            .max(metrics.min_frame_extent),
    )
}
