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
fn grayscale_glyph_placement_keeps_alpha_subpixel_bins_for_editor_labels() {
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
fn grayscale_share_bin_uses_alpha_subpixel_bins() {
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
