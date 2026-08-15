use super::super::super::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract) fn fill_pixel_span(
    span: &mut [u8],
    color: [u8; 4],
) {
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
pub(in crate::ui::retained_host::host_contract) fn write_pixel(
    frame: &mut HostRgbaFrame,
    x: u32,
    y: u32,
    color: [u8; 4],
) {
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

#[inline]
pub(in crate::ui::retained_host::host_contract) fn write_pixel_with_coverage(
    frame: &mut HostRgbaFrame,
    x: u32,
    y: u32,
    color: [u8; 4],
    coverage: f32,
) {
    if !coverage.is_finite() || coverage <= 0.0 || color[3] == 0 {
        return;
    }
    if coverage >= 1.0 {
        write_pixel(frame, x, y, color);
        return;
    }
    let mut covered_color = color;
    covered_color[3] = (color[3] as f32 * coverage.clamp(0.0, 1.0)).round() as u8;
    write_pixel(frame, x, y, covered_color);
}
