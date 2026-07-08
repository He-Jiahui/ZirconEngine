pub(in crate::ui::retained_host::host_contract::paint_text::draw) const RETAINED_TEXT_SUBPIXEL_BINS: u8 =
    8;

const FALLBACK_TEXT_ORIGIN_PX: f32 = 0.0;

pub(super) fn finite_text_origin(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        FALLBACK_TEXT_ORIGIN_PX
    }
}

pub(super) fn screen_pixel_x(screen_x: f32) -> i32 {
    let units = rounded_subpixel_units(screen_x);
    units.div_euclid(RETAINED_TEXT_SUBPIXEL_BINS as i32)
}

pub(super) fn screen_subpixel_bin(screen_x: f32) -> u8 {
    let units = rounded_subpixel_units(screen_x);
    units.rem_euclid(RETAINED_TEXT_SUBPIXEL_BINS as i32) as u8
}

pub(super) fn subpixel_offset_for_bin(bin: u8) -> f32 {
    bin as f32 / RETAINED_TEXT_SUBPIXEL_BINS as f32
}

pub(super) fn quantized_left_offset_px(offset: f32) -> f32 {
    if !offset.is_finite() {
        return FALLBACK_TEXT_ORIGIN_PX;
    }

    let bins = RETAINED_TEXT_SUBPIXEL_BINS as f32;
    (offset * bins).round() / bins
}

fn rounded_subpixel_units(screen_x: f32) -> i32 {
    (finite_text_origin(screen_x) * RETAINED_TEXT_SUBPIXEL_BINS as f32).round() as i32
}

#[cfg(test)]
mod tests {
    use super::{
        finite_text_origin, quantized_left_offset_px, screen_pixel_x, screen_subpixel_bin,
        subpixel_offset_for_bin, FALLBACK_TEXT_ORIGIN_PX, RETAINED_TEXT_SUBPIXEL_BINS,
    };

    #[test]
    fn placement_metrics_drop_non_finite_origins_to_fallback_origin() {
        assert_eq!(finite_text_origin(f32::NAN), FALLBACK_TEXT_ORIGIN_PX);
        assert_eq!(finite_text_origin(f32::INFINITY), FALLBACK_TEXT_ORIGIN_PX);
    }

    #[test]
    fn placement_metrics_project_screen_x_to_pixel_and_bin() {
        assert_eq!(screen_pixel_x(20.875), 20);
        assert_eq!(screen_subpixel_bin(20.875), 7);
        assert_eq!(screen_subpixel_bin(20.75), RETAINED_TEXT_SUBPIXEL_BINS - 2);
    }

    #[test]
    fn placement_metrics_round_high_fraction_into_next_pixel() {
        assert_eq!(screen_pixel_x(20.90), 20);
        assert_eq!(screen_subpixel_bin(20.90), RETAINED_TEXT_SUBPIXEL_BINS - 1);
        assert_eq!(screen_pixel_x(20.95), 21);
        assert_eq!(screen_subpixel_bin(20.95), 0);
    }

    #[test]
    fn placement_metrics_keep_negative_origins_on_same_quantized_grid() {
        assert_eq!(screen_pixel_x(-0.20), -1);
        assert_eq!(screen_subpixel_bin(-0.20), RETAINED_TEXT_SUBPIXEL_BINS - 2);
        assert_eq!(screen_pixel_x(-0.95), -1);
        assert_eq!(screen_subpixel_bin(-0.95), 0);
    }

    #[test]
    fn placement_metrics_project_bins_back_to_offsets() {
        assert_eq!(subpixel_offset_for_bin(0), 0.0);
        assert_eq!(subpixel_offset_for_bin(4), 0.5);
        assert_eq!(subpixel_offset_for_bin(7), 0.875);
    }

    #[test]
    fn placement_metrics_quantize_left_offset_to_phase_grid() {
        assert_eq!(quantized_left_offset_px(-0.30), -0.25);
        assert_eq!(quantized_left_offset_px(0.30), 0.25);
        assert_eq!(quantized_left_offset_px(f32::NAN), FALLBACK_TEXT_ORIGIN_PX);
    }
}
