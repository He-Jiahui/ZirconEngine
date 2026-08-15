use super::*;
use std::time::Instant;

#[test]
fn measured_width_sums_source_subranges() {
    let style = test_style();
    let shaped = shape_unconstrained_line("Wi", &style);
    let line = shaped.lines.first().expect("shaped line");
    let first = measured_width(&shaped, 0, 1, true);
    let second = measured_width(&shaped, 1, 2, true);
    let full = measured_width(&shaped, 0, 2, true);

    assert!(first > 0.0);
    assert!(second > 0.0);
    assert!((first + second - full).abs() < 0.1);
    assert!((full - line.measured_width).abs() < 0.1);
}

#[test]
fn measured_width_uses_absolute_source_ranges() {
    let style = test_style();
    let source = "xxWi";
    let shaped = shape_horizontal_line(
        &source[2..],
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 2,
            end: source.len(),
        },
    );
    let local_shaped = shape_unconstrained_line("Wi", &style);

    assert_eq!(measured_width(&shaped, 0, 2, true), 0.0);
    assert!(
        (measured_width(&shaped, 2, 3, true) - measured_width(&local_shaped, 0, 1, true)).abs()
            < 0.1
    );
    assert!(
        (measured_width(&shaped, 2, source.len(), true)
            - measured_width(&local_shaped, 0, 2, true))
        .abs()
            < 0.1
    );
}

#[test]
fn measured_width_splits_partial_cluster_by_grapheme_count() {
    let style = test_style();
    let shaped = shape_unconstrained_line("fi", &style);
    let first = measured_width(&shaped, 0, 1, true);
    let second = measured_width(&shaped, 1, 2, true);
    let full = measured_width(&shaped, 0, 2, true);

    assert!(first > 0.0);
    assert!(second > 0.0);
    assert!((first + second - full).abs() < 0.1);
}

#[test]
fn measured_grapheme_widths_projects_visual_glyphs_into_source_order() {
    let shaped = ShapedGlyphRun {
        source_text: std::sync::Arc::from("abc"),
        source_range: TextRange { start: 0, end: 3 },
        direction: TextDirection::LeftToRight,
        orientation: crate::text::TextOrientation::Horizontal,
        vertical_mode: crate::text::VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 50.0,
        measured_height: 12.0,
        lines: vec![crate::text::ShapedTextLine {
            line_index: 0,
            source_range: TextRange { start: 0, end: 3 },
            visual_range: TextRange { start: 0, end: 3 },
            measured_width: 50.0,
            baseline: 9.0,
            line_height: 12.0,
            glyphs: vec![test_glyph(2, 3, 30.0), test_glyph(0, 2, 20.0)],
        }],
    };

    assert_eq!(
        measured_grapheme_widths_from_shaped(&shaped, "abc"),
        vec![10.0, 10.0, 30.0]
    );
}

#[test]
fn measured_grapheme_widths_distributes_a_multi_grapheme_cluster_once_per_glyph() {
    let shaped = ShapedGlyphRun {
        source_text: std::sync::Arc::from("abcd"),
        source_range: TextRange { start: 0, end: 4 },
        direction: TextDirection::LeftToRight,
        orientation: crate::text::TextOrientation::Horizontal,
        vertical_mode: crate::text::VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 40.0,
        measured_height: 12.0,
        lines: vec![crate::text::ShapedTextLine {
            line_index: 0,
            source_range: TextRange { start: 0, end: 4 },
            visual_range: TextRange { start: 0, end: 4 },
            measured_width: 40.0,
            baseline: 9.0,
            line_height: 12.0,
            glyphs: vec![test_glyph(0, 4, 40.0)],
        }],
    };

    assert_eq!(
        measured_grapheme_widths_from_shaped(&shaped, "abcd"),
        vec![10.0, 10.0, 10.0, 10.0]
    );
}

#[test]
fn measured_grapheme_widths_shapes_the_complete_text_once() {
    let shaped = Arc::new(ShapedGlyphRun {
        source_text: std::sync::Arc::from("abcd"),
        source_range: TextRange { start: 0, end: 4 },
        direction: TextDirection::LeftToRight,
        orientation: crate::text::TextOrientation::Horizontal,
        vertical_mode: crate::text::VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 40.0,
        measured_height: 12.0,
        lines: vec![crate::text::ShapedTextLine {
            line_index: 0,
            source_range: TextRange { start: 0, end: 4 },
            visual_range: TextRange { start: 0, end: 4 },
            measured_width: 40.0,
            baseline: 9.0,
            line_height: 12.0,
            glyphs: vec![test_glyph(0, 4, 40.0)],
        }],
    });
    let mut provider = CountingShapeRunProvider {
        shaped,
        shape_calls: 0,
    };

    assert_eq!(
        measured_grapheme_widths_with_provider("abcd", &test_style(), &mut provider),
        vec![10.0, 10.0, 10.0, 10.0]
    );
    assert_eq!(provider.shape_calls, 1);
}

#[test]
#[ignore = "records machine-local Text03 grapheme projection p50/p95 evidence"]
fn grapheme_projection_scale_evidence_reports_p50_p95() {
    const SAMPLE_COUNT: usize = 31;
    let inputs = [
        ("latin_combining", "a\u{0301}"),
        ("rtl_arabic_mark", "\u{0645}\u{0651}"),
    ];
    let grapheme_counts = [1_usize, 100, 1_000, 10_000];

    #[cfg(feature = "profiling")]
    let expected_profile_span_count = SAMPLE_COUNT * inputs.len() * grapheme_counts.len();
    #[cfg(feature = "profiling")]
    let _profile_capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    #[cfg(feature = "profiling")]
    {
        let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
        config.session_id = "text03-grapheme-projection-scale".to_owned();
        config.max_spans = expected_profile_span_count.saturating_mul(2);
        assert!(crate::core::diagnostics::profiling::start_capture(config).active);
    }

    for (case_name, grapheme) in inputs {
        for grapheme_count in grapheme_counts {
            let source = grapheme.repeat(grapheme_count);
            let shaped = shape_unconstrained_line(&source, &test_style());
            let glyph_count = shaped
                .lines
                .iter()
                .map(|line| line.glyphs.len())
                .sum::<usize>();
            let mut samples = Vec::with_capacity(SAMPLE_COUNT);

            for _ in 0..SAMPLE_COUNT {
                let started = Instant::now();
                let advances = measured_grapheme_widths_from_shaped(&shaped, &source);
                samples.push(started.elapsed().as_nanos());

                assert_eq!(advances.len(), grapheme_count);
                assert!(advances
                    .iter()
                    .all(|advance| advance.is_finite() && *advance >= 0.0));
            }

            samples.sort_unstable();
            let p50 = samples[samples.len() / 2];
            let p95 = samples[samples.len() * 95 / 100];
            println!(
                "text03_grapheme_projection_scale case={case_name} graphemes={grapheme_count} glyphs={glyph_count} p50_ns={p50} p95_ns={p95}"
            );
        }
    }

    #[cfg(feature = "profiling")]
    {
        let snapshot = crate::core::diagnostics::profiling::snapshot();
        let mut projection_durations = snapshot
            .spans
            .iter()
            .filter(|span| span.category == "text.measure" && span.name == "grapheme_projection")
            .map(|span| span.duration_us)
            .collect::<Vec<_>>();
        assert_eq!(projection_durations.len(), expected_profile_span_count);
        projection_durations.sort_unstable();
        let p50 = projection_durations[projection_durations.len() / 2];
        let p95 = projection_durations[projection_durations.len() * 95 / 100];
        println!(
            "text03_grapheme_projection_profile spans={} p50_us={p50} p95_us={p95}",
            projection_durations.len()
        );
        assert!(
            !crate::core::diagnostics::profiling::reset_capture().active,
            "Text03 profiling capture must be reset before other tests run"
        );
    }
}

#[test]
fn measure_text_size_preserves_a_trailing_empty_line() {
    let style = test_style();
    let line_metrics = line_metrics_with_provider(&style, &mut DirectTextShapeRunProvider);
    let measured = measure_text_size("line\n", &style);

    assert!((measured.height - line_metrics.line_height * 2.0).abs() < 0.1);
}

#[test]
fn measure_text_size_preserves_consecutive_trailing_empty_lines() {
    let style = test_style();
    let line_metrics = line_metrics_with_provider(&style, &mut DirectTextShapeRunProvider);
    let measured = measure_text_size("line\n\n", &style);

    assert!((measured.height - line_metrics.line_height * 3.0).abs() < 0.1);
}

#[test]
fn measure_text_size_uses_all_mandatory_unicode_separators() {
    let style = test_style();
    let line_metrics = line_metrics_with_provider(&style, &mut DirectTextShapeRunProvider);
    let measured = measure_text_size("line\r\n\u{2028}", &style);

    assert!((measured.height - line_metrics.line_height * 3.0).abs() < 0.1);
    assert!((measured.width - measure_line_width("line", &style)).abs() < 0.1);
}

#[test]
fn measure_source_range_can_request_unkerned_backend_shape() {
    let style = test_style();
    let range = TextRange { start: 0, end: 2 };
    let kerned = shape_unconstrained_line_with_kerning("AV", &style, true);
    let unkerned = shape_unconstrained_line_with_kerning("AV", &style, false);
    let kerned_width = measure_text_source_range_width_with_kerning("AV", &style, range, true);
    let unkerned_width = measure_text_source_range_width_with_kerning("AV", &style, range, false);

    assert!(kerned.include_kerning);
    assert!(!unkerned.include_kerning);
    assert!((kerned_width - measured_width(&kerned, 0, 2, true)).abs() < 0.1);
    assert!((unkerned_width - measured_width(&unkerned, 0, 2, false)).abs() < 0.1);
}

fn test_style() -> TextStyle {
    TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    }
}

fn test_glyph(source_start: usize, source_end: usize, advance: f32) -> ShapedGlyph {
    ShapedGlyph {
        glyph_id: 0,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange {
            start: source_start,
            end: source_end,
        },
        visual_range: TextRange {
            start: source_start,
            end: source_end,
        },
        advance,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: crate::text::ShapedGlyphClusterFlags::default(),
        rotation: crate::text::ShapedGlyphRotation::None,
        script: crate::text::ShapedGlyphScript::default(),
    }
}

struct CountingShapeRunProvider {
    shaped: Arc<ShapedGlyphRun>,
    shape_calls: usize,
}

impl TextShapeRunProvider for CountingShapeRunProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        _text: &str,
        _style: &TextStyle,
        _direction: TextDirection,
        _source_range: TextRange,
        _include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        self.shape_calls += 1;
        Arc::clone(&self.shaped)
    }
}
