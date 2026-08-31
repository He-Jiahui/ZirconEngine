const MIN_PHYSICAL_RASTER_PX: f32 = 1.0;
const MIN_SAMPLE_OFFSET_X: f32 = 0.0;
const MAX_SUBPIXEL_SAMPLE_OFFSET_X: f32 = 0.999;
const MAX_FALLBACK_SAMPLE_OFFSET_X: f32 = 1.999;
const RETAINED_COMPACT_TEXT_UNHINTED_MAX_PX: f32 = 13.0;
const MISSING_FONTDUE_Y_OFFSET_PX: f32 = 0.0;

pub(super) const NATIVE_RASTER_SAMPLE_SCALE: f32 = 1.0;
pub(super) const NATIVE_SWASH_SAMPLE_OFFSET_X: f32 = 0.0;
pub(super) const NATIVE_SWASH_SAMPLE_OFFSET_Y: f32 = 0.0;

pub(super) fn physical_raster_px_size(logical_px: f32, surface_scale_factor: f32) -> u32 {
    let logical_px = finite_positive_or_default(logical_px, MIN_PHYSICAL_RASTER_PX);
    let surface_scale_factor = finite_positive_or_default(surface_scale_factor, 1.0);
    (logical_px * surface_scale_factor)
        .round()
        .max(MIN_PHYSICAL_RASTER_PX) as u32
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

pub(super) fn swash_hinting_for_physical_size(physical_px: f32) -> bool {
    physical_px > RETAINED_COMPACT_TEXT_UNHINTED_MAX_PX
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

fn finite_positive_or_default(value: f32, default_value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        default_value
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fontdue_fallback_sample_offset_x, missing_fontdue_y_offset,
        normalized_fallback_sample_offset_x, normalized_subpixel_offset, physical_raster_px_size,
        swash_hinting_for_physical_size, NATIVE_RASTER_SAMPLE_SCALE, NATIVE_SWASH_SAMPLE_OFFSET_X,
        NATIVE_SWASH_SAMPLE_OFFSET_Y,
    };

    #[test]
    fn raster_metrics_quantize_logical_size_into_physical_ppem() {
        assert_eq!(physical_raster_px_size(13.0, 1.0), 13);
        assert_eq!(physical_raster_px_size(13.0, 1.25), 16);
        assert_eq!(physical_raster_px_size(13.0, 1.5), 20);
        assert_eq!(physical_raster_px_size(13.0, 2.0), 26);
    }

    #[test]
    fn raster_metrics_default_invalid_inputs_without_rejecting_valid_downscale() {
        assert_eq!(physical_raster_px_size(0.0, 1.0), 1);
        assert_eq!(physical_raster_px_size(13.0, f32::NAN), 13);
        assert_eq!(physical_raster_px_size(13.0, 0.5), 7);
    }

    #[test]
    fn native_raster_sampling_metrics_are_explicit() {
        assert_eq!(NATIVE_RASTER_SAMPLE_SCALE, 1.0);
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
        assert!(!swash_hinting_for_physical_size(10.0));
        assert!(!swash_hinting_for_physical_size(13.0));
        assert!(swash_hinting_for_physical_size(13.01));
        assert_eq!(missing_fontdue_y_offset(), 0.0);
    }
}
