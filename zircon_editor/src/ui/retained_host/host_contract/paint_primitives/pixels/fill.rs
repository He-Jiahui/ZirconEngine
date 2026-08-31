use super::super::super::data::FrameRect;
use super::super::super::paint_color::{
    blend_premultiplied_linear_srgb_pixel, srgb_byte_to_linear,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::geometry::{
    clamped_corner_radius, inset_frame, interval_pixel_coverage, rect_pixel_coverage,
    rounded_rect_pixel_coverage,
};
use super::span::{fill_pixel_span, write_pixel_with_coverage};

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

pub(in crate::ui::retained_host::host_contract) fn fill_rect_pixel_coverage(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    color: [u8; 4],
) {
    let full_x0 = rect.x.ceil().max(0.0) as u32;
    let full_x1 = (rect.x + rect.width).floor().max(0.0) as u32;
    for y in target.y0..target.y1 {
        let y_coverage = interval_pixel_coverage(y, rect.y, rect.y + rect.height);
        if y_coverage < 1.0 - f32::EPSILON || full_x0 >= full_x1 {
            write_rect_coverage_range(frame, target.x0, target.x1, y, rect, color);
            continue;
        }

        let interior_x0 = full_x0.clamp(target.x0, target.x1);
        let interior_x1 = full_x1.clamp(target.x0, target.x1);
        write_rect_coverage_range(frame, target.x0, interior_x0, y, rect, color);
        if interior_x0 < interior_x1 {
            fill_pixel_rect(
                frame,
                &PixelRect {
                    x0: interior_x0,
                    y0: y,
                    x1: interior_x1,
                    y1: y + 1,
                },
                color,
            );
        }
        write_rect_coverage_range(frame, interior_x1, target.x1, y, rect, color);
    }
}

fn write_rect_coverage_range(
    frame: &mut HostRgbaFrame,
    x0: u32,
    x1: u32,
    y: u32,
    rect: &FrameRect,
    color: [u8; 4],
) {
    for x in x0..x1 {
        write_pixel_with_coverage(frame, x, y, color, rect_pixel_coverage(x, y, rect));
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
            let coverage = rounded_rect_pixel_coverage(x, y, rect, corner_radius);
            write_pixel_with_coverage(frame, x, y, color, coverage);
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn fill_rounded_box_pixels(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    fill_color: [u8; 4],
    border_color: [u8; 4],
    border_width: f32,
    corner_radius: f32,
) {
    let inner = inset_frame(rect, border_width);
    let inner_radius = (clamped_corner_radius(rect, corner_radius) - border_width).max(0.0);
    for y in target.y0..target.y1 {
        for x in target.x0..target.x1 {
            // Fill and border partition one outer coverage; they must never source-over each other.
            let outer_coverage = rounded_rect_pixel_coverage(x, y, rect, corner_radius);
            let inner_coverage =
                rounded_rect_pixel_coverage(x, y, &inner, inner_radius).min(outer_coverage);
            let border_coverage = outer_coverage - inner_coverage;
            let fill_alpha = f32::from(fill_color[3]) / 255.0 * inner_coverage;
            let border_alpha = f32::from(border_color[3]) / 255.0 * border_coverage;
            let output_alpha = fill_alpha + border_alpha;
            if output_alpha <= 0.0 {
                continue;
            }
            let premultiplied_linear = std::array::from_fn(|channel| {
                srgb_byte_to_linear(fill_color[channel]) * fill_alpha
                    + srgb_byte_to_linear(border_color[channel]) * border_alpha
            });
            let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
            blend_premultiplied_linear_srgb_pixel(
                &mut frame.as_bytes_mut()[offset..offset + 4],
                premultiplied_linear,
                output_alpha,
            );
        }
    }
}
