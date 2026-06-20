use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::geometry::{inset_frame, rounded_rect_contains_pixel};
use super::span::write_pixel;

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
    let inner_radius = (corner_radius - border_width).max(0.0);
    for y in target.y0..target.y1 {
        for x in target.x0..target.x1 {
            if !rounded_rect_contains_pixel(x, y, rect, corner_radius) {
                continue;
            }
            if has_inner && rounded_rect_contains_pixel(x, y, &inner, inner_radius) {
                continue;
            }
            write_pixel(frame, x, y, color);
        }
    }
}
