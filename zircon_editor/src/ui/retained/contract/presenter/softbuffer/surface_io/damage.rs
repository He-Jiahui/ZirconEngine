use std::num::NonZeroU32;

use softbuffer::Rect;

use super::super::super::super::data::FrameRect;

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
