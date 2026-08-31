use super::super::super::paint_color::blend_srgb_pixel_linear;
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

    for pixel in span.chunks_exact_mut(4) {
        blend_srgb_pixel_linear(pixel, color, 1.0);
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

    blend_srgb_pixel_linear(&mut bytes[offset..offset + 4], color, 1.0);
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
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    blend_srgb_pixel_linear(
        &mut frame.as_bytes_mut()[offset..offset + 4],
        color,
        coverage,
    );
}
