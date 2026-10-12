use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::geometry::{
    clamped_corner_radius, inset_frame, interval_pixel_coverage, rect_pixel_coverage,
    rounded_rect_pixel_coverage,
};
use super::span::write_pixel_with_coverage;

pub(in crate::ui::retained_host::host_contract) fn fill_rect_border_pixels(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    color: [u8; 4],
    border_width: f32,
) {
    let inner = inset_frame(rect, border_width);
    if !is_visible_frame(&inner) {
        for y in target.y0..target.y1 {
            write_rect_border_range(frame, target.x0, target.x1, y, rect, None, color);
        }
        return;
    }

    let full_inner_x0 = inner.x.ceil().max(0.0) as u32;
    let full_inner_x1 = (inner.x + inner.width).floor().max(0.0) as u32;
    for y in target.y0..target.y1 {
        let inner_y_coverage = interval_pixel_coverage(y, inner.y, inner.y + inner.height);
        if inner_y_coverage < 1.0 || full_inner_x0 >= full_inner_x1 {
            write_rect_border_range(frame, target.x0, target.x1, y, rect, Some(&inner), color);
            continue;
        }

        let left_end = full_inner_x0.clamp(target.x0, target.x1);
        let right_start = full_inner_x1.clamp(target.x0, target.x1);
        write_rect_border_range(frame, target.x0, left_end, y, rect, Some(&inner), color);
        write_rect_border_range(frame, right_start, target.x1, y, rect, Some(&inner), color);
    }
}

fn write_rect_border_range(
    frame: &mut HostRgbaFrame,
    x0: u32,
    x1: u32,
    y: u32,
    rect: &FrameRect,
    inner: Option<&FrameRect>,
    color: [u8; 4],
) {
    for x in x0..x1 {
        let outer_coverage = rect_pixel_coverage(x, y, rect);
        let inner_coverage = inner
            .map(|inner| rect_pixel_coverage(x, y, inner))
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
