use super::super::super::data::FrameRect;

pub(super) fn full_surface_pixels(size: (u32, u32)) -> u64 {
    u64::from(size.0.max(1)) * u64::from(size.1.max(1))
}

pub(super) fn damage_pixel_count(frame: &FrameRect, size: (u32, u32)) -> u64 {
    let x0 = frame.x.floor().max(0.0).min(size.0.max(1) as f32) as u32;
    let y0 = frame.y.floor().max(0.0).min(size.1.max(1) as f32) as u32;
    let x1 = (frame.x + frame.width)
        .ceil()
        .max(0.0)
        .min(size.0.max(1) as f32) as u32;
    let y1 = (frame.y + frame.height)
        .ceil()
        .max(0.0)
        .min(size.1.max(1) as f32) as u32;
    u64::from(x1.saturating_sub(x0)) * u64::from(y1.saturating_sub(y0))
}

pub(super) fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}
