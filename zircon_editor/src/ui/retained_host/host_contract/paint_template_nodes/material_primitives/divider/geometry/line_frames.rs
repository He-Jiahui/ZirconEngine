use super::super::super::super::super::data::FrameRect;
use super::align::pixel_aligned;
use super::metrics::DIVIDER_THICKNESS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_line_y(
    rect: &FrameRect,
) -> f32 {
    pixel_aligned(rect.y + (rect.height - DIVIDER_THICKNESS).max(0.0) * 0.5)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vertical_line_x(
    rect: &FrameRect,
) -> f32 {
    pixel_aligned(rect.x + (rect.width - DIVIDER_THICKNESS).max(0.0) * 0.5)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_line_frame(
    left: f32,
    right: f32,
    y: f32,
) -> Option<FrameRect> {
    let width = right - left;
    (width > 0.5).then(|| FrameRect {
        x: left,
        y,
        width,
        height: DIVIDER_THICKNESS,
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vertical_line_frame(
    x: f32,
    top: f32,
    bottom: f32,
) -> Option<FrameRect> {
    let height = bottom - top;
    (height > 0.5).then(|| FrameRect {
        x,
        y: top,
        width: DIVIDER_THICKNESS,
        height,
    })
}
