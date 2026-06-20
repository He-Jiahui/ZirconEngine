use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Rect, Surface};
use winit::window::Window;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;

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

pub(in crate::ui::retained_host::host_contract) fn softbuffer_damage_rect(
    frame: Option<&FrameRect>,
    size: (u32, u32),
) -> Option<Rect> {
    let frame = frame?;
    let (x0, y0, x1, y1) = pixel_bounds(frame, size)?;
    Some(Rect {
        x: x0,
        y: y0,
        width: NonZeroU32::new(x1.saturating_sub(x0))?,
        height: NonZeroU32::new(y1.saturating_sub(y0))?,
    })
}

pub(in crate::ui::retained_host::host_contract) fn pixel_bounds(
    frame: &FrameRect,
    size: (u32, u32),
) -> Option<(u32, u32, u32, u32)> {
    let x0 = frame.x.floor().max(0.0).min(size.0 as f32) as u32;
    let y0 = frame.y.floor().max(0.0).min(size.1 as f32) as u32;
    let x1 = (frame.x + frame.width).ceil().max(0.0).min(size.0 as f32) as u32;
    let y1 = (frame.y + frame.height).ceil().max(0.0).min(size.1 as f32) as u32;
    (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
}

pub(in crate::ui::retained_host::host_contract) fn damage_pixel_count(
    frame: &FrameRect,
    size: (u32, u32),
) -> u64 {
    pixel_bounds(frame, size)
        .map(|(x0, y0, x1, y1)| x1.saturating_sub(x0) as u64 * y1.saturating_sub(y0) as u64)
        .unwrap_or(0)
}

pub(in crate::ui::retained_host::host_contract) fn current_window_size(
    window: &dyn Window,
) -> (u32, u32) {
    let size = window.surface_size();
    clamp_size((size.width, size.height))
}

pub(in crate::ui::retained_host::host_contract) fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.0), non_zero(size.1))
}

pub(in crate::ui::retained_host::host_contract) fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}
