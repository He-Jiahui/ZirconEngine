use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::damage::pixel_bounds;

pub(in crate::ui::retained_host::host_contract) fn copy_rgba_to_softbuffer(
    frame: &HostRgbaFrame,
    buffer: &mut [u32],
    damage: Option<&FrameRect>,
    size: (u32, u32),
) {
    let (x0, y0, x1, y1) = damage
        .and_then(|damage| pixel_bounds(damage, size))
        .unwrap_or((0, 0, size.0, size.1));
    let width = size.0 as usize;
    let frame_bytes = frame.as_bytes();
    let copy_width = x1.saturating_sub(x0) as usize;
    for y in y0..y1 {
        let pixel_start = y as usize * width + x0 as usize;
        let pixel_end = pixel_start + copy_width;
        let byte_start = pixel_start * 4;
        let byte_end = pixel_end * 4;
        let source_row = &frame_bytes[byte_start..byte_end];
        let target_row = &mut buffer[pixel_start..pixel_end];
        for (pixel, rgba) in target_row.iter_mut().zip(source_row.chunks_exact(4)) {
            let red = rgba[0] as u32;
            let green = rgba[1] as u32;
            let blue = rgba[2] as u32;
            *pixel = (red << 16) | (green << 8) | blue;
        }
    }
}
