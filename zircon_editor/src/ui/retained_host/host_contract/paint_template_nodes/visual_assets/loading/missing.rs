use super::super::{HostPaintImagePixels, RasterTargetSize, ICON_TINT};
use super::cache::{cached_visual_asset_pixels, store_visual_asset_pixels};
use super::key::image_pixels_cache_key;

const MISSING_ICON_SMALL_SAMPLES_PER_AXIS: u32 = 4;
const MISSING_ICON_LARGE_SAMPLES_PER_AXIS: u32 = 2;
const MISSING_ICON_SMALL_MAX_EDGE: u32 = 32;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn missing_icon_pixels(
    base_key: &str,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let color = tint.unwrap_or(ICON_TINT);
    let cache_key = missing_icon_cache_key(base_key, target, color);
    if let Some(cached) = cached_visual_asset_pixels(&cache_key) {
        return cached;
    }
    let mut rgba = vec![0; target.width as usize * target.height as usize * 4];
    let edge = target.width.min(target.height);
    let stroke = (edge / 10).clamp(1, 3) as f32;
    let samples_per_axis = if target.width.max(target.height) <= MISSING_ICON_SMALL_MAX_EDGE {
        MISSING_ICON_SMALL_SAMPLES_PER_AXIS
    } else {
        MISSING_ICON_LARGE_SAMPLES_PER_AXIS
    };

    for y in 0..target.height {
        for x in 0..target.width {
            if !missing_icon_pixel_may_be_covered(x, y, target, edge as f32, stroke) {
                continue;
            }
            let coverage =
                missing_icon_sample_coverage(x, y, target, edge as f32, stroke, samples_per_axis);
            if coverage == 0 {
                continue;
            }
            let offset = ((y * target.width + x) as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&scale_alpha_by_coverage(color, coverage));
        }
    }

    let image = HostPaintImagePixels {
        resource_key: cache_key.clone(),
        width: target.width,
        height: target.height,
        rgba: rgba.into(),
        atlas: None,
    };
    let image = image.is_valid().then_some(image);
    store_visual_asset_pixels(cache_key, base_key, std::iter::empty(), image.clone());
    image
}

fn missing_icon_pixel_may_be_covered(
    x: u32,
    y: u32,
    target: RasterTargetSize,
    edge: f32,
    stroke: f32,
) -> bool {
    let width = target.width as f32;
    let height = target.height as f32;
    let border = (x as f32) < stroke
        || (y as f32) < stroke
        || width - ((x + 1) as f32) < stroke
        || height - ((y + 1) as f32) < stroke;
    if border {
        return true;
    }

    let normalized_x = (x as f32 + 0.5) / width;
    let normalized_y = (y as f32 + 0.5) / height;
    let pixel_coverage_margin = stroke + 1.0;
    (normalized_x - normalized_y).abs() * edge < pixel_coverage_margin
        || (normalized_x + normalized_y - 1.0).abs() * edge < pixel_coverage_margin
}

fn missing_icon_sample_coverage(
    x: u32,
    y: u32,
    target: RasterTargetSize,
    edge: f32,
    stroke: f32,
    samples_per_axis: u32,
) -> u8 {
    let mut covered_samples = 0;
    for sample_y in 0..samples_per_axis {
        for sample_x in 0..samples_per_axis {
            let px = x as f32 + (sample_x as f32 + 0.5) / samples_per_axis as f32;
            let py = y as f32 + (sample_y as f32 + 0.5) / samples_per_axis as f32;
            if missing_icon_sample_is_covered(px, py, target, edge, stroke) {
                covered_samples += 1;
            }
        }
    }
    let sample_count = samples_per_axis * samples_per_axis;
    ((covered_samples * 255 + sample_count / 2) / sample_count) as u8
}

fn missing_icon_sample_is_covered(
    px: f32,
    py: f32,
    target: RasterTargetSize,
    edge: f32,
    stroke: f32,
) -> bool {
    let width = target.width as f32;
    let height = target.height as f32;
    let border = px < stroke || py < stroke || width - px < stroke || height - py < stroke;
    let normalized_x = px / width;
    let normalized_y = py / height;
    let diagonal = (normalized_x - normalized_y).abs() * edge < stroke
        || (normalized_x + normalized_y - 1.0).abs() * edge < stroke;
    border || diagonal
}

fn scale_alpha_by_coverage(mut color: [u8; 4], coverage: u8) -> [u8; 4] {
    color[3] = ((u16::from(color[3]) * u16::from(coverage) + 127) / 255) as u8;
    color
}

fn missing_icon_cache_key(base_key: &str, target: RasterTargetSize, color: [u8; 4]) -> String {
    image_pixels_cache_key(
        &format!("missing-icon:{base_key}"),
        Some(target),
        Some(color),
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::RasterTargetSize;
    use super::{
        missing_icon_cache_key, missing_icon_pixel_may_be_covered, missing_icon_pixels,
        missing_icon_sample_coverage, MISSING_ICON_LARGE_SAMPLES_PER_AXIS,
        MISSING_ICON_SMALL_MAX_EDGE, MISSING_ICON_SMALL_SAMPLES_PER_AXIS,
    };

    #[test]
    fn missing_icon_cache_key_separates_tint_variants() {
        let target = RasterTargetSize::new(16, 16).expect("valid raster size");

        assert_ne!(
            missing_icon_cache_key("icon:save", target, [1, 2, 3, 4]),
            missing_icon_cache_key("icon:save", target, [4, 3, 2, 1]),
        );
    }

    #[test]
    fn missing_icon_diagonal_contains_fractional_edge_coverage() {
        let image = missing_icon_pixels(
            "icon:fractional-edge-coverage",
            RasterTargetSize::new(16, 16).expect("valid raster size"),
            Some([20, 30, 40, 255]),
        )
        .expect("visible fallback");

        assert!(
            image
                .rgba
                .chunks_exact(4)
                .any(|pixel| (1..=254).contains(&pixel[3])),
            "the diagonal fallback must retain fractional device-pixel coverage"
        );
    }

    #[test]
    fn large_missing_icon_rejects_pixels_far_from_the_sparse_strokes() {
        let target = RasterTargetSize::new(4096, 4096).expect("valid raster size");

        assert!(missing_icon_pixel_may_be_covered(
            2048, 2048, target, 4096.0, 3.0
        ));
        assert!(!missing_icon_pixel_may_be_covered(
            2048, 1024, target, 4096.0, 3.0
        ));
    }

    #[test]
    fn sparse_stroke_rejection_never_discards_covered_samples() {
        for width in [1, 16, 33, 64] {
            for height in [1, 15, 32, 65] {
                let target = RasterTargetSize::new(width, height).expect("valid raster size");
                let edge = width.min(height);
                let stroke = (edge / 10).clamp(1, 3) as f32;
                let samples_per_axis = if width.max(height) <= MISSING_ICON_SMALL_MAX_EDGE {
                    MISSING_ICON_SMALL_SAMPLES_PER_AXIS
                } else {
                    MISSING_ICON_LARGE_SAMPLES_PER_AXIS
                };
                for y in 0..height {
                    for x in 0..width {
                        let coverage = missing_icon_sample_coverage(
                            x,
                            y,
                            target,
                            edge as f32,
                            stroke,
                            samples_per_axis,
                        );
                        assert!(
                            coverage == 0
                                || missing_icon_pixel_may_be_covered(
                                    x,
                                    y,
                                    target,
                                    edge as f32,
                                    stroke,
                                ),
                            "{width}x{height} pixel ({x}, {y}) lost coverage {coverage}"
                        );
                    }
                }
            }
        }
    }
}
