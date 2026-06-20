use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::geometry::rounded_rect_contains_pixel;
use super::span::{fill_pixel_span, write_pixel};

pub(in crate::ui::retained_host::host_contract) fn fill_pixel_rect(
    frame: &mut HostRgbaFrame,
    rect: &PixelRect,
    color: [u8; 4],
) {
    let frame_width = frame.width() as usize;
    let x0 = rect.x0 as usize;
    let x1 = rect.x1 as usize;
    let bytes = frame.as_bytes_mut();

    for y in rect.y0 as usize..rect.y1 as usize {
        let row_start = ((y * frame_width) + x0) * 4;
        let row_end = ((y * frame_width) + x1) * 4;
        fill_pixel_span(&mut bytes[row_start..row_end], color);
    }
}

pub(in crate::ui::retained_host::host_contract) fn fill_rounded_pixel_rect(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    color: [u8; 4],
    corner_radius: f32,
) {
    for y in target.y0..target.y1 {
        for x in target.x0..target.x1 {
            if rounded_rect_contains_pixel(x, y, rect, corner_radius) {
                write_pixel(frame, x, y, color);
            }
        }
    }
}
