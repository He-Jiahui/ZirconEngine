use std::sync::Arc;

use super::*;
use crate::core::framework::text::TextDirection;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};

#[test]
fn measured_line_reuses_one_shape_for_advances_and_face_metrics() {
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let expected = shape_unconstrained_line("\u{4e16}\u{754c}", &style);
    let expected_line = expected
        .lines
        .first()
        .expect("direct shape must publish a line");
    let mut provider = CountingShapeRunProvider::default();

    let measured = measure_line_with_provider("\u{4e16}\u{754c}", &style, &mut provider)
        .into_result()
        .expect("measure one final physical line");

    assert_eq!(provider.shape_calls, 1);
    assert_eq!(measured.grapheme_advances.len(), 2);
    assert!((measured.metrics.baseline - expected_line.baseline).abs() < 0.01);
    assert!((measured.metrics.line_height - expected_line.line_height).abs() < 0.01);
}

#[test]
fn text_size_sums_the_metrics_of_each_physical_line() {
    let style = TextStyle::default();
    let mut provider = PhysicalLineMetricsProvider::default();

    let measured = measure_text_size_with_provider("alpha\nbeta", &style, &mut provider)
        .into_result()
        .expect("measure physical text lines");

    assert_eq!(provider.shaped_texts, vec!["alpha", "beta"]);
    assert!((measured.height - 30.0).abs() < 0.01);
}

#[test]
fn text_size_keeps_multiline_height_finite_when_f32_accumulation_overflows() {
    let style = TextStyle::default();
    let mut provider = PhysicalLineMetricsProvider::default();

    let measured = measure_text_size_with_provider("max\nmax", &style, &mut provider)
        .into_result()
        .expect("measure extreme physical text lines");

    assert!(measured.height.is_finite());
    assert_eq!(measured.height, f32::MAX);
}

#[test]
fn measured_source_range_width_keeps_extreme_glyph_sum_finite() {
    let source_range = TextRange { start: 0, end: 2 };
    let glyph = |start, end| ShapedGlyph {
        glyph_id: 0,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange { start, end },
        visual_range: TextRange { start, end },
        advance: f32::MAX,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: crate::text::ShapedGlyphClusterFlags::default(),
        rotation: crate::text::ShapedGlyphRotation::None,
        script: crate::text::ShapedGlyphScript::default(),
    };
    let shaped = ShapedGlyphRun {
        source_text: Arc::from("ab"),
        source_range,
        unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
        primary_face_id: None,
        direction: TextDirection::LeftToRight,
        orientation: crate::text::TextOrientation::Horizontal,
        vertical_mode: crate::text::VerticalMode::Mixed,
        include_kerning: true,
        measured_width: f32::MAX,
        measured_height: 1.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![crate::text::ShapedHardLine {
            line_index: 0,
            source_range,
            visual_range: source_range,
            measured_width: f32::MAX,
            baseline: 1.0,
            line_height: 1.0,
            glyphs: vec![glyph(0, 1), glyph(1, 2)],
        }],
    };

    let width = measured_width(&shaped, 0, 2, true);

    assert!(width.is_finite());
    assert_eq!(width, f32::MAX);
}

#[derive(Default)]
struct CountingShapeRunProvider {
    direct: DirectTextShapeRunProvider,
    shape_calls: usize,
}

impl TextShapeRunProvider for CountingShapeRunProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> crate::text::shaping::TextShapingOutcome {
        self.shape_calls = self.shape_calls.saturating_add(1);
        self.direct.shape_horizontal_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }
}

#[derive(Default)]
struct PhysicalLineMetricsProvider {
    shaped_texts: Vec<String>,
}

impl TextShapeRunProvider for PhysicalLineMetricsProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        _style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> crate::text::shaping::TextShapingOutcome {
        self.shaped_texts.push(text.to_owned());
        let (baseline, line_height) = match text {
            "alpha" => (7.0, 10.0),
            "beta" => (14.0, 20.0),
            "max" => (f32::MAX, f32::MAX),
            unexpected => panic!("unexpected physical-line measurement: {unexpected:?}"),
        };
        let width = text.len() as f32;
        TextShapingOutcome::Ready(Arc::new(ShapedGlyphRun {
            source_text: Arc::from(text),
            source_range,
            unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
            primary_face_id: None,
            direction,
            orientation: crate::text::TextOrientation::Horizontal,
            vertical_mode: crate::text::VerticalMode::Mixed,
            include_kerning,
            measured_width: width,
            measured_height: line_height,
            horizontal_composition_receipt: None,
            horizontal_line_raw_metrics: Vec::new(),
            horizontal_glyph_metric_spans: Vec::new(),
            lines: vec![crate::text::ShapedHardLine {
                line_index: 0,
                source_range,
                visual_range: source_range,
                measured_width: width,
                baseline,
                line_height,
                glyphs: Vec::new(),
            }],
        }))
    }
}
