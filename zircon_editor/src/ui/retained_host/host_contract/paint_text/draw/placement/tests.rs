use super::*;
use crate::ui::retained_host::host_contract::paint_theme::HostTextSmoothing;

#[test]
fn retained_glyph_left_offset_px_quantizes_without_floor_bias() {
    assert_eq!(retained_glyph_left_offset_px(-0.30), -0.25);
    assert_eq!(retained_glyph_left_offset_px(0.30), 0.25);
    assert_eq!(retained_glyph_left_offset_px(0.80), 0.75);
}

#[test]
fn retained_glyph_left_offset_px_drops_non_finite_values() {
    assert_eq!(retained_glyph_left_offset_px(f32::NAN), 0.0);
    assert_eq!(retained_glyph_left_offset_px(f32::INFINITY), 0.0);
}

#[test]
fn retained_text_origin_device_px_snaps_line_origin_without_floor_bias() {
    assert_eq!(retained_text_origin_device_px(8.49), 8.0);
    assert_eq!(retained_text_origin_device_px(8.50), 9.0);
    assert_eq!(retained_text_origin_device_px(8.875), 9.0);
}

#[test]
fn retained_text_origin_device_px_drops_non_finite_values() {
    assert_eq!(retained_text_origin_device_px(f32::NAN), 0.0);
    assert_eq!(retained_text_origin_device_px(f32::INFINITY), 0.0);
}

#[test]
fn retained_text_origin_for_smoothing_snaps_only_grayscale_origins() {
    assert_eq!(
        retained_text_origin_for_smoothing(8.875, HostTextSmoothing::Grayscale),
        9.0
    );
    assert_eq!(
        retained_text_origin_for_smoothing(8.875, HostTextSmoothing::Subpixel),
        8.875
    );
}

#[test]
fn retained_text_origin_for_smoothing_drops_non_finite_values() {
    assert_eq!(
        retained_text_origin_for_smoothing(f32::NAN, HostTextSmoothing::Grayscale),
        0.0
    );
    assert_eq!(
        retained_text_origin_for_smoothing(f32::INFINITY, HostTextSmoothing::Subpixel),
        0.0
    );
}

#[test]
fn grayscale_glyph_placement_uses_subpixel_bins_after_line_origin_snap() {
    let left = retained_glyph_placement_for_smoothing(20.24, HostTextSmoothing::Grayscale);
    let right = retained_glyph_placement_for_smoothing(20.51, HostTextSmoothing::Grayscale);

    assert_eq!(left.pixel_x, 20);
    assert!((left.subpixel_offset - 0.25).abs() < 0.001);
    assert_eq!(right.pixel_x, 20);
    assert!((right.subpixel_offset - 0.5).abs() < 0.001);
}

#[test]
fn subpixel_glyph_placement_keeps_eight_phase_bins() {
    let placement = retained_glyph_placement_for_smoothing(20.51, HostTextSmoothing::Subpixel);

    assert_eq!(placement.pixel_x, 20);
    assert!((placement.subpixel_offset - 0.5).abs() < 0.001);
}

#[test]
fn retained_glyph_placement_rounds_upper_phase_to_nearest_pixel() {
    let near_right = RetainedGlyphPlacement::from_screen_x(20.90);
    let upper_edge = RetainedGlyphPlacement::from_screen_x(20.95);

    assert_eq!(near_right.pixel_x, 20);
    assert_eq!(upper_edge.pixel_x, 21);
    assert!((near_right.subpixel_offset - 0.875).abs() < 0.001);
    assert_eq!(upper_edge.subpixel_offset, 0.0);
}

#[test]
fn retained_glyph_placement_keeps_high_fraction_within_nearest_quantized_error() {
    let baseline = RetainedGlyphPlacement::from_screen_x(44.875);
    let nearby = RetainedGlyphPlacement::from_screen_x(44.95);

    assert_eq!(baseline.pixel_x, 44);
    assert_eq!(nearby.pixel_x, 45);
    assert!((baseline.subpixel_offset - 0.875).abs() < 0.001);
    assert_eq!(nearby.subpixel_offset, 0.0);
}

#[test]
fn retained_glyph_placement_bounds_horizontal_quantization_error() {
    for fraction in [
        0.01_f32, 0.06, 0.12, 0.19, 0.27, 0.43, 0.56, 0.69, 0.82, 0.94, 0.99,
    ] {
        let x = 20.0 + fraction;
        let placement = RetainedGlyphPlacement::from_screen_x(x);
        let quantized_x = placement.pixel_x as f32 + placement.subpixel_offset;

        assert!(
            (quantized_x - x).abs() <= 0.0625,
            "retained glyph placement should use the nearest 1/8px phase without a high-phase clamp bias: x={x}, placement={placement:?}"
        );
    }
}

#[test]
fn grayscale_share_bin_uses_subpixel_bins_after_line_origin_snap() {
    assert!(!retained_glyph_placements_share_bin_for_smoothing(
        20.12,
        20.20,
        HostTextSmoothing::Grayscale,
    ));
    assert!(retained_glyph_placements_share_bin_for_smoothing(
        20.49,
        20.51,
        HostTextSmoothing::Grayscale,
    ));
    assert!(!retained_glyph_placements_share_bin_for_smoothing(
        20.51,
        20.88,
        HostTextSmoothing::Grayscale,
    ));
}

#[test]
fn subpixel_share_bin_keeps_eight_phase_bins() {
    assert!(!retained_glyph_placements_share_bin_for_smoothing(
        20.12,
        20.20,
        HostTextSmoothing::Subpixel,
    ));
    assert!(retained_glyph_placements_share_bin_for_smoothing(
        20.49,
        20.51,
        HostTextSmoothing::Subpixel,
    ));
}
