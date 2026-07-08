use super::super::super::super::data::FrameRect;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn min_frame_extent() -> f32 {
    command_palette_metrics().min_frame_extent
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    let min_frame_extent = min_frame_extent();
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(min_frame_extent),
        height: rect.height.round().max(min_frame_extent),
    }
}

pub(super) fn symmetric_extent(inset: f32) -> f32 {
    inset * 2.0
}

pub(super) fn centered_offset(outer: f32, inner: f32) -> f32 {
    (outer - inner).max(0.0) * 0.5
}
