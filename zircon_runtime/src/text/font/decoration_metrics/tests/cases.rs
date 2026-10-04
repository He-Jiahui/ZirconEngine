use std::fs;
use std::path::Path;

use super::{
    text_decoration_frame, TextDecorationKind, TextDecorationMetrics, TextDecorationMetricsCache,
    MIN_VISIBLE_TEXT_DECORATION_PX,
};
use crate::asset::{FontAssetFaceMetrics, FontAssetLineMetrics};
use crate::core::framework::text::TextWritingMode;
use crate::text::font::FontDatabase;
use crate::text::TextFrame;

#[test]
fn render_text_decoration_metrics_from_face_tables() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let bytes = fs::read(&source).unwrap();
    let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
    let mut database = FontDatabase::default();
    let font_face = database
        .register_font_file(&source, Some("Decoration Metrics"), 0)
        .unwrap();
    let metrics = TextDecorationMetricsCache::default().resolve(&database, font_face, 40.0);
    let unit_scale = 40.0 / f32::from(face.units_per_em());
    let underline = face.underline_metrics().unwrap();
    let strikeout = face.strikeout_metrics().unwrap();

    assert_close(
        metrics.underline.position_px,
        f32::from(underline.position) * unit_scale,
    );
    assert_close(
        metrics.underline.thickness_px,
        f32::from(underline.thickness).abs() * unit_scale,
    );
    assert_close(
        metrics.strikeout.position_px,
        f32::from(strikeout.position) * unit_scale,
    );
    assert_close(
        metrics.strikeout.thickness_px,
        f32::from(strikeout.thickness).abs() * unit_scale,
    );
}

#[test]
fn render_text_decoration_underline_geometry() {
    let metrics = synthetic_metrics(20.0);
    let horizontal = text_decoration_frame(
        TextFrame::new(10.0, 20.0, 100.0, 30.0),
        42.0,
        TextWritingMode::HorizontalTopToBottom,
        metrics,
        TextDecorationKind::Underline,
    );
    assert_eq!(horizontal.x, 10.0);
    assert_eq!(horizontal.width, 100.0);
    assert_close(horizontal.y, 43.5);
    assert_close(horizontal.height, 1.0);

    let vertical = text_decoration_frame(
        TextFrame::new(10.0, 20.0, 30.0, 100.0),
        25.0,
        TextWritingMode::VerticalRightToLeft,
        metrics,
        TextDecorationKind::Underline,
    );
    assert_close(vertical.x, 26.5);
    assert_close(vertical.width, 1.0);
    assert_eq!(vertical.y, 20.0);
    assert_eq!(vertical.height, 100.0);
}

#[test]
fn render_text_decoration_strikeout_fallback_and_scale() {
    let fallback = TextDecorationMetrics::fallback(20.0);
    assert_close(fallback.underline.position_px, -2.0);
    assert_close(fallback.underline.thickness_px, 1.0);
    assert_close(fallback.strikeout.position_px, 6.0);
    assert_close(fallback.strikeout.thickness_px, 1.0);

    let small = synthetic_metrics(20.0);
    let large = synthetic_metrics(40.0);
    assert_close(
        large.underline.position_px,
        small.underline.position_px * 2.0,
    );
    assert_close(
        large.strikeout.thickness_px,
        small.strikeout.thickness_px * 2.0,
    );

    let strike = text_decoration_frame(
        TextFrame::new(10.0, 20.0, 100.0, 30.0),
        42.0,
        TextWritingMode::HorizontalTopToBottom,
        small,
        TextDecorationKind::Strikethrough,
    );
    assert_close(strike.y, 35.5);
    assert_close(strike.height, MIN_VISIBLE_TEXT_DECORATION_PX);
}

fn synthetic_metrics(display_px: f32) -> TextDecorationMetrics {
    TextDecorationMetrics::from_font_units(
        FontAssetFaceMetrics {
            units_per_em: 1_000,
            ascender: 800,
            underline: Some(FontAssetLineMetrics {
                position: -100,
                thickness: 50,
            }),
            strikeout: Some(FontAssetLineMetrics {
                position: 300,
                thickness: 40,
            }),
            ..FontAssetFaceMetrics::default()
        },
        display_px,
    )
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "actual={actual} expected={expected}"
    );
}
