use super::super::super::super::data::FrameRect;
use super::super::super::visual_assets::HostPaintImagePixels;
use std::sync::Arc;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn apply_rounded_alpha_mask(
    image: &mut HostPaintImagePixels,
    corner_radius: f32,
    rect: &FrameRect,
) {
    let mask_radius = rounded_alpha_mask_radius(image, corner_radius, rect);
    if mask_radius <= 0.0 {
        return;
    }

    let width = image.width;
    let height = image.height;
    let rgba = Arc::make_mut(&mut image.rgba);
    for y in 0..height {
        for x in 0..width {
            if rounded_mask_contains_pixel(x, y, width, height, mask_radius) {
                continue;
            }
            let offset = ((y as usize * width as usize) + x as usize) * 4 + 3;
            rgba[offset] = 0;
        }
    }
    image.resource_key = format!(
        "mui-avatar-mask:{}x{}:{:.3}:{}",
        image.width, image.height, mask_radius, image.resource_key
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn rounded_alpha_mask_radius(
    image: &HostPaintImagePixels,
    corner_radius: f32,
    rect: &FrameRect,
) -> f32 {
    if corner_radius <= 0.0 || image.width == 0 || image.height == 0 {
        return 0.0;
    }
    let display_edge = rect.width.min(rect.height).max(1.0);
    let mask_edge = image.width.min(image.height) as f32;
    (corner_radius / display_edge * mask_edge).clamp(0.0, mask_edge * 0.5)
}

fn rounded_mask_contains_pixel(x: u32, y: u32, width: u32, height: u32, radius: f32) -> bool {
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let right = width as f32;
    let bottom = height as f32;
    let radius = radius.min(right.min(bottom) * 0.5).max(0.0);
    if radius <= 0.0 {
        return px >= 0.0 && px < right && py >= 0.0 && py < bottom;
    }
    let center_x = clamp_to_ordered_range(px, radius, right - radius);
    let center_y = clamp_to_ordered_range(py, radius, bottom - radius);
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
