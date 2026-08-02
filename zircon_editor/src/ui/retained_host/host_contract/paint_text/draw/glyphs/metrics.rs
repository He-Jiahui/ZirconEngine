use std::ops::Range;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) const TEXT_RASTER_SUPERSAMPLE: f32 = 8.0;

const MIN_GLYPH_RASTER_SCALE: f32 = 1.0;
const MIN_COMBINED_SAMPLE_OFFSET_X: f32 = 0.0;
const MAX_COMBINED_SAMPLE_OFFSET_X: f32 = 1.999;
const THIN_STROKE_HIGH_SAMPLE: u8 = 220;
const THIN_STROKE_MAX_AVERAGE: u8 = 96;
const THIN_STROKE_MIN_COVERAGE: u8 = 128;
const ITALIC_ROW_SKEW_MAX_PX: f32 = 2.0;
const SINGLE_GLYPH_DRAW_PASS_COUNT: i32 = 1;

pub(super) fn logical_raster_extent(
    raster_extent: usize,
    raster_scale: f32,
    sample_offset: f32,
) -> usize {
    let raster_scale = glyph_raster_scale(raster_scale);
    let sample_offset = combined_sample_offset_x(sample_offset);
    (raster_extent as f32 / raster_scale + sample_offset).ceil() as usize
}

pub(super) fn uses_native_pixel_sampling(
    raster_width: usize,
    raster_height: usize,
    raster_scale: f32,
) -> bool {
    raster_width == 0
        || raster_height == 0
        || !raster_scale.is_finite()
        || raster_scale <= MIN_GLYPH_RASTER_SCALE
}

pub(super) fn raster_sample_x_range(
    logical_column: usize,
    raster_scale: f32,
    sample_offset_x: f32,
    raster_width: usize,
) -> Range<usize> {
    let sample_offset_x = combined_sample_offset_x(sample_offset_x);
    let x0 = (((logical_column as f32) - sample_offset_x) * raster_scale).floor() as isize;
    let x1 = ((((logical_column + 1) as f32) - sample_offset_x) * raster_scale).ceil() as isize;

    x0.max(0) as usize..x1.max(0).min(raster_width as isize) as usize
}

pub(super) fn raster_sample_y_range(
    logical_row: usize,
    raster_scale: f32,
    raster_height: usize,
) -> Range<usize> {
    let y0 = ((logical_row as f32) * raster_scale).floor() as usize;
    let y1 = (((logical_row + 1) as f32) * raster_scale).ceil() as usize;

    y0..y1.min(raster_height)
}

pub(super) fn averaged_channel_coverage(sum: u32, count: u32, max_coverage: u8) -> u8 {
    let average = ((sum as f32 / count as f32).round()).min(255.0) as u8;
    thin_stroke_preserved_coverage(average, max_coverage)
}

pub(super) fn thin_stroke_preserved_coverage(average: u8, max_coverage: u8) -> u8 {
    if average > 0 && average < THIN_STROKE_MAX_AVERAGE && max_coverage >= THIN_STROKE_HIGH_SAMPLE {
        THIN_STROKE_MIN_COVERAGE.max(average)
    } else {
        average
    }
}

pub(super) fn italic_pixel_offset(style: UiTextRunPaintStyle, row: usize, height: usize) -> i32 {
    if !style.emphasis || height == 0 {
        return 0;
    }
    let top_bias = height.saturating_sub(row) as f32 / height.max(1) as f32;
    (top_bias * ITALIC_ROW_SKEW_MAX_PX).round() as i32
}

pub(super) fn glyph_draw_pass_count(_style: UiTextRunPaintStyle) -> i32 {
    SINGLE_GLYPH_DRAW_PASS_COUNT
}

fn glyph_raster_scale(raster_scale: f32) -> f32 {
    if raster_scale.is_finite() && raster_scale > MIN_GLYPH_RASTER_SCALE {
        raster_scale
    } else {
        MIN_GLYPH_RASTER_SCALE
    }
}

pub(super) fn combined_sample_offset_x(sample_offset: f32) -> f32 {
    if sample_offset.is_finite() {
        sample_offset.clamp(MIN_COMBINED_SAMPLE_OFFSET_X, MAX_COMBINED_SAMPLE_OFFSET_X)
    } else {
        MIN_COMBINED_SAMPLE_OFFSET_X
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_COMBINED_SAMPLE_OFFSET_X, MIN_GLYPH_RASTER_SCALE, averaged_channel_coverage,
        combined_sample_offset_x, glyph_draw_pass_count, glyph_raster_scale, italic_pixel_offset,
        logical_raster_extent, raster_sample_x_range, raster_sample_y_range,
        thin_stroke_preserved_coverage, uses_native_pixel_sampling,
    };
    use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

    #[test]
    fn glyph_raster_scale_keeps_supersample_scale_above_minimum() {
        assert_eq!(glyph_raster_scale(8.0), 8.0);
        assert_eq!(glyph_raster_scale(1.0), MIN_GLYPH_RASTER_SCALE);
        assert_eq!(glyph_raster_scale(f32::NAN), MIN_GLYPH_RASTER_SCALE);
    }

    #[test]
    fn combined_sample_offset_clamps_to_known_range() {
        assert_eq!(combined_sample_offset_x(-0.25), 0.0);
        assert_eq!(combined_sample_offset_x(4.0), MAX_COMBINED_SAMPLE_OFFSET_X);
        assert_eq!(combined_sample_offset_x(f32::NAN), 0.0);
    }

    #[test]
    fn logical_extent_projects_scaled_raster_with_fractional_tail() {
        assert_eq!(logical_raster_extent(16, 8.0, 1.125), 4);
        assert_eq!(logical_raster_extent(4, 1.0, 0.0), 4);
    }

    #[test]
    fn sample_ranges_project_logical_pixel_to_raster_window() {
        assert_eq!(raster_sample_x_range(0, 4.0, 0.5, 8), 0..2);
        assert_eq!(raster_sample_x_range(1, 4.0, 1.5, 8), 0..2);
        assert_eq!(raster_sample_y_range(0, 4.0, 8), 0..4);
    }

    #[test]
    fn native_sampling_predicate_handles_empty_or_unscaled_rasters() {
        assert!(uses_native_pixel_sampling(0, 4, 4.0));
        assert!(uses_native_pixel_sampling(4, 4, 1.0));
        assert!(uses_native_pixel_sampling(4, 4, f32::NAN));
        assert!(!uses_native_pixel_sampling(4, 4, 4.0));
    }

    #[test]
    fn coverage_metrics_preserve_thin_strokes_after_downsampling() {
        assert_eq!(thin_stroke_preserved_coverage(64, 255), 128);
        assert_eq!(thin_stroke_preserved_coverage(128, 255), 128);
        assert_eq!(averaged_channel_coverage(255, 4, 255), 128);
    }

    #[test]
    fn glyph_style_metrics_project_italic_offset_and_single_draw_pass() {
        assert_eq!(
            italic_pixel_offset(
                UiTextRunPaintStyle {
                    emphasis: true,
                    ..UiTextRunPaintStyle::default()
                },
                0,
                4,
            ),
            2
        );
        assert_eq!(italic_pixel_offset(UiTextRunPaintStyle::default(), 0, 4), 0);
        assert_eq!(glyph_draw_pass_count(UiTextRunPaintStyle::default()), 1);
    }
}
