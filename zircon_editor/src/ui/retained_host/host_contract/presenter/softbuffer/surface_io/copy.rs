use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_color::{linear_to_srgb_byte, srgb_byte_to_linear};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::damage::pixel_bounds;

#[derive(Clone, Copy)]
struct AxisSample {
    lower: usize,
    upper: usize,
    upper_weight: f32,
}

fn axis_sample(source_extent: usize, target_extent: usize, target_index: usize) -> AxisSample {
    // Sample at the target pixel center, then clamp to both source edges.
    // This keeps native-resize snapshots smooth without exposing an
    // uninitialized border pixel or allocating a per-frame lookup table.
    let source_position =
        ((target_index as f32 + 0.5) * source_extent as f32 / target_extent as f32 - 0.5)
            .clamp(0.0, source_extent.saturating_sub(1) as f32);
    let lower = source_position.floor() as usize;
    let upper = (lower + 1).min(source_extent.saturating_sub(1));
    AxisSample {
        lower,
        upper,
        upper_weight: source_position - lower as f32,
    }
}

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

pub(in crate::ui::retained_host::host_contract) fn copy_scaled_rgba_to_softbuffer(
    source: &HostRgbaFrame,
    buffer: &mut [u32],
    target_size: (u32, u32),
) {
    let (target_width, target_height) = target_size;
    let source_width = source.width();
    let source_height = source.height();
    let target_pixel_count = target_width as usize * target_height as usize;
    let source_pixel_count = source_width as usize * source_height as usize;
    if target_width == 0
        || target_height == 0
        || source_width == 0
        || source_height == 0
        || buffer.len() < target_pixel_count
        || source.as_bytes().len() < source_pixel_count.saturating_mul(4)
    {
        return;
    }

    let source_bytes = source.as_bytes();
    let source_width = source_width as usize;
    let source_height = source_height as usize;
    let target_width = target_width as usize;
    let target_height = target_height as usize;
    if source_width == target_width && source_height == target_height {
        copy_rgba_to_softbuffer(source, buffer, None, target_size);
        return;
    }

    for target_y in 0..target_height {
        let y_sample = axis_sample(source_height, target_height, target_y);
        for target_x in 0..target_width {
            let x_sample = axis_sample(source_width, target_width, target_x);
            let red = bilinear_channel(source_bytes, source_width, &y_sample, &x_sample, 0);
            let green = bilinear_channel(source_bytes, source_width, &y_sample, &x_sample, 1);
            let blue = bilinear_channel(source_bytes, source_width, &y_sample, &x_sample, 2);
            buffer[target_y * target_width + target_x] =
                (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue);
        }
    }
}

fn bilinear_channel(
    source: &[u8],
    source_width: usize,
    y: &AxisSample,
    x: &AxisSample,
    channel: usize,
) -> u8 {
    let top_left = source[(y.lower * source_width + x.lower) * 4 + channel];
    let top_right = source[(y.lower * source_width + x.upper) * 4 + channel];
    let bottom_left = source[(y.upper * source_width + x.lower) * 4 + channel];
    let bottom_right = source[(y.upper * source_width + x.upper) * 4 + channel];
    let top = linear_lerp(top_left, top_right, x.upper_weight);
    let bottom = linear_lerp(bottom_left, bottom_right, x.upper_weight);
    linear_to_srgb_byte(linear_lerp_f32(top, bottom, y.upper_weight))
}

fn linear_lerp(left: u8, right: u8, weight: f32) -> f32 {
    linear_lerp_f32(
        srgb_byte_to_linear(left),
        srgb_byte_to_linear(right),
        weight,
    )
}

fn linear_lerp_f32(left: f32, right: f32, weight: f32) -> f32 {
    left + (right - left) * weight.clamp(0.0, 1.0)
}
