use super::super::super::super::data::FrameRect;
use super::super::super::visual_assets::HostPaintImagePixels;
use std::sync::Arc;

const AVATAR_MASK_SAMPLES_PER_AXIS: u32 = 8;
const PIXEL_HALF_DIAGONAL: f32 = std::f32::consts::FRAC_1_SQRT_2;

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
mod tests {
    use super::apply_rounded_alpha_mask;
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::paint_frame::{
        HostPaintAtlasImage, HostPaintImageUvRect,
    };
    use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::HostPaintImagePixels;

    fn image(alpha: u8, atlas: Option<HostPaintAtlasImage>) -> HostPaintImagePixels {
        let mut rgba = vec![255; 8 * 8 * 4];
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[3] = alpha;
        }
        HostPaintImagePixels {
            resource_key: "avatar-source".to_owned(),
            width: 8,
            height: 8,
            rgba: rgba.into(),
            atlas,
        }
    }

    #[test]
    fn rounded_avatar_mask_uses_fractional_coverage_and_preserves_source_alpha() {
        let mut image = image(128, None);

        apply_rounded_alpha_mask(
            &mut image,
            4.0,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
        );

        let alpha = image
            .rgba
            .chunks_exact(4)
            .map(|pixel| pixel[3])
            .collect::<Vec<_>>();
        assert!(alpha.contains(&0));
        assert!(alpha.contains(&128));
        assert!(alpha.iter().any(|value| *value > 0 && *value < 128));
    }

    #[test]
    fn rounded_avatar_mask_invalidates_the_unmasked_atlas_fast_path() {
        let atlas = HostPaintAtlasImage {
            resource_key: "avatar-atlas".to_owned(),
            resource_generation: 7,
            width: 64,
            height: 64,
            rgba: None,
            uv: HostPaintImageUvRect {
                min: [0.0, 0.0],
                max: [0.5, 0.5],
            },
        };
        let mut image = image(255, Some(atlas));

        apply_rounded_alpha_mask(
            &mut image,
            4.0,
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
        );

        assert!(image.atlas.is_none());
    }
}
