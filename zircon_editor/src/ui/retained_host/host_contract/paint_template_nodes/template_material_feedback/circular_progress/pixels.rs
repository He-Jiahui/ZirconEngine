use std::f32::consts::PI;

const CIRCULAR_THICKNESS_FACTOR: f32 = 0.16;
const CIRCULAR_THICKNESS_MIN: f32 = 3.0;
const CIRCULAR_THICKNESS_MAX: f32 = 6.0;

pub(super) fn circular_progress_pixels(
    size: u32,
    percent: f32,
    track: [u8; 4],
    fill: [u8; 4],
) -> Vec<u8> {
    let mut rgba = vec![0; size as usize * size as usize * 4];
    let center = size as f32 * 0.5;
    let radius = (size as f32 * 0.5 - 0.5).max(1.0);
    let thickness = (size as f32 * CIRCULAR_THICKNESS_FACTOR)
        .clamp(CIRCULAR_THICKNESS_MIN, CIRCULAR_THICKNESS_MAX);
    let inner = (radius - thickness).max(0.0);
    let percent = percent.clamp(0.0, 1.0);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < inner || distance > radius {
                continue;
            }
            let angle = dy.atan2(dx);
            let turn = ((angle + PI * 0.5).rem_euclid(PI * 2.0)) / (PI * 2.0);
            let color = if turn <= percent { fill } else { track };
            let offset = ((y as usize * size as usize) + x as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
    rgba
}
