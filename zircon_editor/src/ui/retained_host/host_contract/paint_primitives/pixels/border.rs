use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::geometry::{clamped_corner_radius, inset_frame, rounded_rect_pixel_coverage};
use super::span::write_pixel_with_coverage;

pub(in crate::ui::retained_host::host_contract) fn fill_rounded_border_pixels(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    color: [u8; 4],
    border_width: f32,
    corner_radius: f32,
) {
    let inner = inset_frame(rect, border_width);
    let has_inner = is_visible_frame(&inner);
    let inner_radius = (clamped_corner_radius(rect, corner_radius) - border_width).max(0.0);
    for y in target.y0..target.y1 {
        for x in target.x0..target.x1 {
            let outer_coverage = rounded_rect_pixel_coverage(x, y, rect, corner_radius);
            let inner_coverage = has_inner
                .then(|| rounded_rect_pixel_coverage(x, y, &inner, inner_radius))
                .unwrap_or(0.0);
            write_pixel_with_coverage(
                frame,
                x,
                y,
                color,
                (outer_coverage - inner_coverage).clamp(0.0, 1.0),
            );
        }
    }
}
