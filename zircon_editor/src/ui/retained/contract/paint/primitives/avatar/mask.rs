use super::super::super::super::data::FrameRect;
use super::super::super::visual_assets::HostPaintImagePixels;
use std::sync::Arc;

const AVATAR_MASK_SAMPLES_PER_AXIS: u32 = 8;
const PIXEL_HALF_DIAGONAL: f32 = std::f32::consts::FRAC_1_SQRT_2;

// 图像路径对头像像素施加与显示框一致的圆角；遮罩后须取消原图集身份并赋予像素资源键。
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
    {
        let rgba = Arc::make_mut(&mut image.rgba);
        for y in 0..height {
            for x in 0..width {
                let coverage = rounded_mask_pixel_coverage(x, y, width, height, mask_radius);
                if coverage >= 1.0 {
                    continue;
                }
                let offset = ((y as usize * width as usize) + x as usize) * 4 + 3;
                let source_alpha = f32::from(rgba[offset]);
                rgba[offset] = (source_alpha * coverage).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    image.atlas = None;
    // BUG: [CR-M18-AVATAR-0001] 资源键仅保留三位小数半径，但遮罩像素和缓存键使用完整 f32；不同遮罩可共享绘制资源键，录制流按同键同代去重后显示旧像素。
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

fn rounded_mask_pixel_coverage(x: u32, y: u32, width: u32, height: u32, radius: f32) -> f32 {
    let center_distance =
        rounded_mask_signed_distance(x as f32 + 0.5, y as f32 + 0.5, width, height, radius);
    if center_distance <= -PIXEL_HALF_DIAGONAL {
        return 1.0;
    }
    if center_distance >= PIXEL_HALF_DIAGONAL {
        return 0.0;
    }

    let mut covered = 0_u32;
    for sample_y in 0..AVATAR_MASK_SAMPLES_PER_AXIS {
        for sample_x in 0..AVATAR_MASK_SAMPLES_PER_AXIS {
            let px = x as f32 + (sample_x as f32 + 0.5) / AVATAR_MASK_SAMPLES_PER_AXIS as f32;
            let py = y as f32 + (sample_y as f32 + 0.5) / AVATAR_MASK_SAMPLES_PER_AXIS as f32;
            covered +=
                u32::from(rounded_mask_signed_distance(px, py, width, height, radius) <= 0.0);
        }
    }
    let sample_count = AVATAR_MASK_SAMPLES_PER_AXIS * AVATAR_MASK_SAMPLES_PER_AXIS;
    covered as f32 / sample_count as f32
}

fn rounded_mask_signed_distance(px: f32, py: f32, width: u32, height: u32, radius: f32) -> f32 {
    let half_width = width as f32 * 0.5;
    let half_height = height as f32 * 0.5;
    let radius = radius.clamp(0.0, half_width.min(half_height));
    let qx = (px - half_width).abs() - (half_width - radius);
    let qy = (py - half_height).abs() - (half_height - radius);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let inside = qx.max(qy).min(0.0);
    outside + inside - radius
}

#[cfg(test)]
#[path = "tests/mask.rs"]
mod tests;
