use super::super::super::RasterTargetSize;
use super::{
    missing_icon_cache_key, missing_icon_pixel_may_be_covered, missing_icon_pixels,
    missing_icon_sample_coverage, MISSING_ICON_LARGE_SAMPLES_PER_AXIS, MISSING_ICON_SMALL_MAX_EDGE,
    MISSING_ICON_SMALL_SAMPLES_PER_AXIS,
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
                            || missing_icon_pixel_may_be_covered(x, y, target, edge as f32, stroke,),
                        "{width}x{height} pixel ({x}, {y}) lost coverage {coverage}"
                    );
                }
            }
        }
    }
}
