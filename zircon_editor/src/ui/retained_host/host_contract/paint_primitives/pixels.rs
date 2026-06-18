use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::{is_visible_frame, PixelRect};

pub(super) fn fill_pixel_rect(frame: &mut HostRgbaFrame, rect: &PixelRect, color: [u8; 4]) {
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

pub(super) fn fill_rounded_pixel_rect(
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

pub(super) fn fill_rounded_border_pixels(
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

pub(super) fn fill_pixel_span(span: &mut [u8], color: [u8; 4]) {
    if color[3] == 255 {
        for pixel in span.chunks_exact_mut(4) {
            write_pixel_channels(pixel, color);
        }
        return;
    }

    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    for pixel in span.chunks_exact_mut(4) {
        blend_pixel(pixel, color, alpha, inverse);
    }
}

#[inline]
fn write_pixel_channels(pixel: &mut [u8], color: [u8; 4]) {
    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];
    pixel[3] = color[3];
}

#[inline]
fn blend_pixel(pixel: &mut [u8], color: [u8; 4], alpha: u32, inverse: u32) {
    for channel in 0..3 {
        let source = color[channel] as u32;
        let destination = pixel[channel] as u32;
        pixel[channel] = ((source * alpha + destination * inverse) / 255) as u8;
    }
    pixel[3] = 255;
}

#[inline]
fn write_pixel(frame: &mut HostRgbaFrame, x: u32, y: u32, color: [u8; 4]) {
    if color[3] == 0 {
        return;
    }
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    let bytes = frame.as_bytes_mut();
    if color[3] == 255 {
        write_pixel_channels(&mut bytes[offset..offset + 4], color);
        return;
    }

    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    blend_pixel(&mut bytes[offset..offset + 4], color, alpha, inverse);
}

pub(super) fn clamped_corner_radius(rect: &FrameRect, corner_radius: f32) -> f32 {
    if !corner_radius.is_finite() {
        return 0.0;
    }
    corner_radius
        .max(0.0)
        .min(rect.width.min(rect.height).max(0.0) * 0.5)
}

fn rounded_rect_contains_pixel(x: u32, y: u32, rect: &FrameRect, corner_radius: f32) -> bool {
    if !is_visible_frame(rect) {
        return false;
    }
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    if px < left || px >= right || py < top || py >= bottom {
        return false;
    }
    let radius = clamped_corner_radius(rect, corner_radius);
    if radius <= 0.0 {
        return true;
    }
    let center_x = clamp_to_ordered_range(px, left + radius, right - radius);
    let center_y = clamp_to_ordered_range(py, top + radius, bottom - radius);
    let dx = px - center_x;
    let dy = py - center_y;
    dx * dx + dy * dy <= radius * radius
}

fn clamp_to_ordered_range(value: f32, min: f32, max: f32) -> f32 {
    if min <= max {
        value.clamp(min, max)
    } else {
        (min + max) * 0.5
    }
}

pub(super) fn inset_frame(rect: &FrameRect, amount: f32) -> FrameRect {
    FrameRect {
        x: rect.x + amount,
        y: rect.y + amount,
        width: (rect.width - amount * 2.0).max(0.0),
        height: (rect.height - amount * 2.0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::clamp_to_ordered_range;

    #[test]
    fn rounded_rect_center_clamp_tolerates_crossed_float_bounds() {
        let clamped = clamp_to_ordered_range(40.0, 40.0, 39.999_992);
        assert!((clamped - 39.999_996).abs() <= f32::EPSILON);
    }
}
