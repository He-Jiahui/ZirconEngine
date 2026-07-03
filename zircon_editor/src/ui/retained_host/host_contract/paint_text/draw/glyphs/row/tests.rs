use super::*;

#[test]
fn sampled_coverage_averages_balanced_supersampled_strokes() {
    let bitmap = [0, 255, 255, 0];

    assert_eq!(sampled_coverage(&bitmap, 2, 2, 0, 0, 2.0, 0.0), 128);
}

#[test]
fn sampled_coverage_preserves_single_pixel_supersampled_strokes() {
    let bitmap = [0, 0, 0, 255];

    assert!(
        sampled_coverage(&bitmap, 2, 2, 0, 0, 2.0, 0.0) >= 128,
        "thin glyph strokes such as underscores should survive logical downsampling"
    );
}

#[test]
fn sampled_coverage_clamps_to_bitmap_edge() {
    let bitmap = [64, 128, 255];

    assert_eq!(sampled_coverage(&bitmap, 3, 1, 1, 0, 2.0, 0.0), 255);
}

#[test]
fn sampled_coverage_applies_fallback_subpixel_phase() {
    let bitmap = [255, 255, 0, 0];

    assert_eq!(sampled_coverage(&bitmap, 4, 1, 0, 0, 4.0, 0.0), 128);
    assert_eq!(sampled_coverage(&bitmap, 4, 1, 0, 0, 4.0, 0.5), 255);
}

#[test]
fn sampled_subpixel_coverage_averages_rgb_channels_independently() {
    let bitmap = [
        0, 30, 60, 0, 120, 150, 180, 0, 60, 90, 120, 0, 180, 210, 240, 0,
    ];

    assert_eq!(
        sampled_subpixel_coverage(&bitmap, 2, 2, 0, 0, 2.0, 0.0),
        [90, 120, 150]
    );
}

#[test]
fn sampled_subpixel_coverage_preserves_native_rgb_channels() {
    let bitmap = [12, 34, 56, 0, 90, 120, 150, 0];

    assert_eq!(
        sampled_subpixel_coverage(&bitmap, 2, 1, 1, 0, 1.0, 0.0),
        [90, 120, 150]
    );
}

#[test]
fn strong_text_uses_selected_font_face_without_synthetic_extra_pass() {
    assert_eq!(
        glyph_draw_pass_count(UiTextRunPaintStyle {
            strong: true,
            ..UiTextRunPaintStyle::default()
        }),
        1
    );
}
