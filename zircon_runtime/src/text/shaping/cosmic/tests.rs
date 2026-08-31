use glyphon::cosmic_text::{FeatureTag, Style as CosmicStyle};

use super::{
    attrs_for_style, cosmic_backend_fallback_allowed, cosmic_line_baseline,
    cosmic_plain_line_starts, cosmic_rich_line_starts, glyph_layout_offset_px,
};
use crate::core::framework::text::TextDirection;
use crate::text::{BackendShapeRequest, OpenTypeFeature, TextOrientation, TextRange, TextStyle};

#[test]
fn glyph_layout_offsets_are_projected_to_pixels() {
    let (x, y) = glyph_layout_offset_px(13.0, 0.25, -0.125);

    assert!((x - 3.25).abs() < 0.001);
    assert!((y + 1.625).abs() < 0.001);
}

#[test]
fn glyph_layout_offsets_drop_non_finite_values() {
    let (x, y) = glyph_layout_offset_px(13.0, f32::NAN, f32::INFINITY);

    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

#[test]
fn attrs_disable_kerning_when_requested() {
    let style = TextStyle::default();
    let attrs = attrs_for_style(BackendShapeRequest::horizontal_with_kerning(
        "AV",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 2 },
        false,
    ));

    assert!(
        attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::KERNING && feature.value == 0)
    );
}

#[test]
fn attrs_apply_italic_style() {
    let style = TextStyle {
        italic: true,
        ..TextStyle::default()
    };
    let attrs = attrs_for_style(BackendShapeRequest::horizontal(
        "Italic",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 6 },
    ));

    assert_eq!(attrs.style, CosmicStyle::Italic);
}

#[test]
fn attrs_apply_normalized_open_type_features() {
    let style = TextStyle::default();
    let features = [
        OpenTypeFeature::new(*b"tnum", 1),
        OpenTypeFeature::new(*b"liga", 0),
    ];
    let request = BackendShapeRequest::horizontal(
        "0123",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 4 },
    )
    .with_features(&features)
    .canonicalized()
    .expect("valid language and feature fixture");
    let attrs = attrs_for_style(request.request());

    assert!(
        attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"tnum") && feature.value == 1)
    );
    assert!(
        attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"liga") && feature.value == 0)
    );
}

#[test]
fn attrs_enable_vertical_substitution_features_for_upright_glyphs() {
    let style = TextStyle::default();
    let attrs = attrs_for_style(BackendShapeRequest::vertical(
        "本文。",
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: "本文。".len(),
        },
        crate::text::VerticalMode::Mixed,
    ));

    assert!(
        attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"vert") && feature.value == 1)
    );
    assert!(
        attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"vrt2") && feature.value == 1)
    );
}

#[test]
fn cosmic_plain_line_starts_follow_line_iter_endings() {
    assert_eq!(cosmic_plain_line_starts(""), vec![0]);
    assert_eq!(cosmic_plain_line_starts("one"), vec![0]);
    assert_eq!(cosmic_plain_line_starts("one\ntwo\n"), vec![0, 4, 8]);
    assert_eq!(cosmic_plain_line_starts("a\rb"), vec![0, 2]);
    assert_eq!(cosmic_plain_line_starts("a\r\nb\n\r"), vec![0, 3, 6]);
    assert_eq!(cosmic_plain_line_starts("a\u{0085}b\u{2029}c"), vec![0]);
}

#[test]
fn cosmic_rich_line_starts_follow_backend_bidi_paragraphs() {
    assert_eq!(cosmic_rich_line_starts(""), vec![0]);
    assert_eq!(cosmic_rich_line_starts("one"), vec![0]);
    assert_eq!(cosmic_rich_line_starts("one\ntwo\n"), vec![0, 4]);
    assert_eq!(cosmic_rich_line_starts("a\rb"), vec![0]);
    assert_eq!(cosmic_rich_line_starts("本\rb"), vec![0, 4]);
    assert_eq!(
        cosmic_rich_line_starts("a\u{0085}b\u{2029}c"),
        vec![0, 3, 7]
    );
    assert_eq!(cosmic_rich_line_starts("a\u{2028}b"), vec![0]);
}

#[test]
fn cosmic_fallback_is_horizontal_only() {
    assert!(cosmic_backend_fallback_allowed(TextOrientation::Horizontal));
    assert!(!cosmic_backend_fallback_allowed(TextOrientation::Vertical));
}

#[test]
fn cosmic_baseline_is_relative_to_each_layout_line() {
    assert_eq!(cosmic_line_baseline(18.0, 10.0, 12.0), 8.0);
    assert_eq!(cosmic_line_baseline(40.0, 24.0, 12.0), 12.0);
    assert_eq!(cosmic_line_baseline(5.0, 9.0, 12.0), 0.0);
}
