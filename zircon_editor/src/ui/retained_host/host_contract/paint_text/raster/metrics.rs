const MIN_RASTER_METRIC_PX: f32 = 1.0;
const MIN_SAMPLE_OFFSET_X: f32 = 0.0;
const MAX_SUBPIXEL_SAMPLE_OFFSET_X: f32 = 0.999;
const MAX_FALLBACK_SAMPLE_OFFSET_X: f32 = 1.999;
const RETAINED_COMPACT_TEXT_UNHINTED_MAX_PX: f32 = 13.0;
const MISSING_FONTDUE_Y_OFFSET_PX: f32 = 0.0;

pub(super) const NATIVE_SWASH_RASTER_SCALE: f32 = MIN_RASTER_METRIC_PX;
pub(super) const NATIVE_SWASH_SAMPLE_OFFSET_X: f32 = 0.0;
pub(super) const NATIVE_SWASH_SAMPLE_OFFSET_Y: f32 = 0.0;

pub(super) fn logical_font_size(logical_px: f32) -> f32 {
    logical_px.max(MIN_RASTER_METRIC_PX)
}

pub(super) fn fallback_raster_scale(raster_scale: f32) -> f32 {
    if raster_scale.is_finite() && raster_scale > MIN_RASTER_METRIC_PX {
        raster_scale
    } else {
        MIN_RASTER_METRIC_PX
    }
}

pub(super) fn fallback_raster_font_size(logical_px: f32, raster_scale: f32) -> f32 {
    logical_font_size(logical_px) * fallback_raster_scale(raster_scale)
}

pub(super) fn raster_metric_scale(raster_scale: f32) -> f32 {
    raster_scale.max(MIN_RASTER_METRIC_PX)
}

pub(super) fn normalized_subpixel_offset(offset: f32) -> f32 {
    normalized_sample_offset(offset, MAX_SUBPIXEL_SAMPLE_OFFSET_X)
}

pub(super) fn normalized_fallback_sample_offset_x(offset: f32) -> f32 {
    normalized_sample_offset(offset, MAX_FALLBACK_SAMPLE_OFFSET_X)
}

pub(super) fn fontdue_fallback_sample_offset_x(
    origin_subpixel_offset: f32,
    raster_left_px: f32,
    x_offset: i32,
) -> f32 {
    normalized_fallback_sample_offset_x(
        normalized_subpixel_offset(origin_subpixel_offset) + raster_left_px - x_offset as f32,
    )
}

pub(super) fn swash_hinting_for_size(logical_px: f32) -> bool {
    logical_px > RETAINED_COMPACT_TEXT_UNHINTED_MAX_PX
}

pub(super) fn missing_fontdue_y_offset() -> f32 {
    MISSING_FONTDUE_Y_OFFSET_PX
}

fn normalized_sample_offset(offset: f32, max_offset: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(MIN_SAMPLE_OFFSET_X, max_offset)
    } else {
        MIN_SAMPLE_OFFSET_X
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MIN_RASTER_METRIC_PX, NATIVE_SWASH_RASTER_SCALE, NATIVE_SWASH_SAMPLE_OFFSET_X,
        NATIVE_SWASH_SAMPLE_OFFSET_Y, fallback_raster_font_size, fallback_raster_scale,
        fontdue_fallback_sample_offset_x, logical_font_size, missing_fontdue_y_offset,
        normalized_fallback_sample_offset_x, normalized_subpixel_offset, raster_metric_scale,
        swash_hinting_for_size,
    };

    #[test]
    fn raster_metrics_preserve_positive_font_size_and_fallback_scale() {
        assert_eq!(logical_font_size(13.0), 13.0);
        assert_eq!(fallback_raster_scale(3.0), 3.0);
        assert_eq!(fallback_raster_font_size(13.0, 3.0), 39.0);
    }

    #[test]
    fn raster_metrics_clamp_invalid_font_size_and_scale() {
        assert_eq!(logical_font_size(0.0), MIN_RASTER_METRIC_PX);
        assert_eq!(fallback_raster_scale(f32::NAN), MIN_RASTER_METRIC_PX);
        assert_eq!(fallback_raster_scale(0.5), MIN_RASTER_METRIC_PX);
        assert_eq!(raster_metric_scale(0.0), MIN_RASTER_METRIC_PX);
    }

    #[test]
    fn native_swash_metrics_are_explicit() {
        assert_eq!(NATIVE_SWASH_RASTER_SCALE, MIN_RASTER_METRIC_PX);
        assert_eq!(NATIVE_SWASH_SAMPLE_OFFSET_X, 0.0);
        assert_eq!(NATIVE_SWASH_SAMPLE_OFFSET_Y, 0.0);
    }

    #[test]
    fn sample_offsets_are_clamped_by_raster_path() {
        assert_eq!(normalized_subpixel_offset(f32::NAN), 0.0);
        assert_eq!(normalized_subpixel_offset(1.5), 0.999);
        assert_eq!(normalized_fallback_sample_offset_x(2.5), 1.999);
        assert_eq!(fontdue_fallback_sample_offset_x(0.75, 2.25, 1), 1.999);
    }

    #[test]
    fn swash_hinting_and_fontdue_y_fallback_are_named() {
        assert!(!swash_hinting_for_size(10.0));
        assert!(!swash_hinting_for_size(13.0));
        assert!(swash_hinting_for_size(13.01));
        assert_eq!(missing_fontdue_y_offset(), 0.0);
    }
}
