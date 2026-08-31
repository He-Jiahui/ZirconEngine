use std::sync::Arc;
use std::time::Instant;

use crate::core::framework::text::TextDirection;
use crate::text::shaping::TextShapeRunProvider;
use crate::text::{
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript,
    ShapedHardLine, TextOrientation, TextRange, TextStyle, VerticalMode,
};
use unicode_segmentation::UnicodeSegmentation;

use super::*;

#[test]
fn long_line_correction_shapes_only_bounded_edge_contexts() {
    let source = format!("A{}Z", "x".repeat(98));
    let units = source
        .graphemes(true)
        .map(|text| BoundaryAdvanceUnit {
            text,
            advance: 10.0,
        })
        .collect::<Vec<_>>();
    let mut provider = EdgeShapeProvider::default();

    let corrected = corrected_line_advance_with_provider(
        &units,
        &TextStyle::default(),
        TextDirection::LeftToRight,
        None,
        &mut provider,
    )
    .into_result()
    .expect("correct line advance");

    assert!((corrected - 1_003.0).abs() < 0.01);
    assert_eq!(provider.shape_calls, 2);
    assert!(
        provider.max_shaped_graphemes <= BOUNDARY_SHAPING_CONTEXT_GRAPHEMES * 2,
        "boundary correction must not rebuild a complete growing line"
    );
}

#[test]
fn boundary_budget_snapshot_matches_the_actual_correction_bounds() {
    let budget = boundary_shaping_budget_snapshot();

    assert_eq!(
        budget.context_graphemes_per_edge,
        BOUNDARY_SHAPING_CONTEXT_GRAPHEMES
    );
    assert_eq!(
        budget.max_reshaped_graphemes,
        BOUNDARY_SHAPING_CONTEXT_GRAPHEMES * 2
    );
    assert_eq!(
        budget.max_correction_steps,
        BOUNDARY_SHAPING_CONTEXT_GRAPHEMES
    );
}

#[test]
fn short_line_correction_shapes_soft_hyphen_suffix_with_the_line_tail() {
    let units = [
        BoundaryAdvanceUnit {
            text: "A",
            advance: 10.0,
        },
        BoundaryAdvanceUnit {
            text: "V",
            advance: 10.0,
        },
    ];
    let mut provider = EdgeShapeProvider::default();

    let corrected = corrected_line_advance_with_provider(
        &units,
        &TextStyle::default(),
        TextDirection::LeftToRight,
        Some("-"),
        &mut provider,
    )
    .into_result()
    .expect("correct soft-hyphen line advance");

    assert!((corrected - 23.0).abs() < 0.01);
    assert_eq!(provider.shape_calls, 1);
    assert_eq!(provider.max_shaped_graphemes, 3);
}

#[test]
fn glyph_range_planner_moves_an_overflowing_corrected_boundary_left() {
    let source = "AVX";
    let index = GraphemeAdvanceIndex::from_metrics(
        source
            .grapheme_indices(true)
            .map(|(source_start, grapheme)| GraphemeAdvanceMetric {
                source_start,
                source_end: source_start + grapheme.len(),
                advance: 10.0,
                cross_extent: 10.0,
            })
            .collect(),
    );
    let mut provider = EdgeShapeProvider::default();

    let ranges = corrected_glyph_ranges_with_provider(
        source,
        &index,
        &TextStyle::default(),
        TextDirection::LeftToRight,
        20.0,
        20.0,
        &mut provider,
    )
    .into_result()
    .expect("plan corrected glyph ranges");

    assert_eq!(ranges, vec![(0, 1), (1, 3)]);
    assert!(provider.max_shaped_graphemes <= BOUNDARY_SHAPING_CONTEXT_GRAPHEMES * 2);
}

#[test]
fn glyph_range_planner_keeps_backend_calls_linear_through_ten_thousand_graphemes() {
    for grapheme_count in [1_usize, 100, 1_000, 10_000] {
        let source = "x".repeat(grapheme_count);
        let index = GraphemeAdvanceIndex::from_metrics(
            source
                .grapheme_indices(true)
                .map(|(source_start, grapheme)| GraphemeAdvanceMetric {
                    source_start,
                    source_end: source_start + grapheme.len(),
                    advance: 10.0,
                    cross_extent: 10.0,
                })
                .collect(),
        );
        let mut provider = EdgeShapeProvider::default();

        let ranges = corrected_glyph_ranges_with_provider(
            &source,
            &index,
            &TextStyle::default(),
            TextDirection::LeftToRight,
            80.0,
            80.0,
            &mut provider,
        )
        .into_result()
        .expect("plan corrected glyph ranges");

        let calls_per_line_limit = (2 * BOUNDARY_SHAPING_CONTEXT_GRAPHEMES + 1).saturating_mul(2);
        assert!(
            provider.shape_calls <= ranges.len().saturating_mul(calls_per_line_limit),
            "{grapheme_count} graphemes exceeded the bounded per-line backend-call budget"
        );
        assert!(
            provider.max_shaped_graphemes <= BOUNDARY_SHAPING_CONTEXT_GRAPHEMES * 2,
            "{grapheme_count} graphemes escaped the bounded shaping window"
        );
    }
}

#[test]
fn indexed_boundary_correction_materializes_only_bounded_edge_units() {
    let source = include_str!("../boundary_correction.rs");

    assert!(source.contains("&metrics[..context_span]"));
    assert!(source.contains("&metrics[metrics.len() - context_span..]"));
    assert!(!source.contains(".get(first..after_last)"));
}

#[test]
#[ignore = "records machine-local Text03 boundary scale p50/p95 evidence"]
fn boundary_scale_evidence_reports_p50_p95() {
    const SAMPLE_COUNT: usize = 31;

    for grapheme_count in [1_usize, 100, 1_000, 10_000] {
        let source = "x".repeat(grapheme_count);
        let index = GraphemeAdvanceIndex::from_metrics(
            source
                .grapheme_indices(true)
                .map(|(source_start, grapheme)| GraphemeAdvanceMetric {
                    source_start,
                    source_end: source_start + grapheme.len(),
                    advance: 10.0,
                    cross_extent: 10.0,
                })
                .collect(),
        );
        let mut durations = Vec::with_capacity(SAMPLE_COUNT);
        let mut backend_calls = 0_usize;
        let mut line_count = 0_usize;
        let mut max_window = 0_usize;
        for _ in 0..SAMPLE_COUNT {
            let mut provider = EdgeShapeProvider::default();
            let started = Instant::now();
            let ranges = corrected_glyph_ranges_with_provider(
                &source,
                &index,
                &TextStyle::default(),
                TextDirection::LeftToRight,
                80.0,
                80.0,
                &mut provider,
            )
            .into_result()
            .expect("plan boundary scale ranges");
            durations.push(started.elapsed().as_nanos());
            backend_calls = provider.shape_calls;
            line_count = ranges.len();
            max_window = provider.max_shaped_graphemes;
        }
        durations.sort_unstable();
        let p50 = durations[durations.len() / 2];
        let p95 = durations[durations.len() * 95 / 100];

        println!(
            "text03_boundary_scale graphemes={grapheme_count} lines={line_count} backend_calls={backend_calls} max_window={max_window} p50_ns={p50} p95_ns={p95}"
        );
        assert!(max_window <= BOUNDARY_SHAPING_CONTEXT_GRAPHEMES * 2);
    }
}

#[derive(Default)]
struct EdgeShapeProvider {
    shape_calls: usize,
    max_shaped_graphemes: usize,
}

impl TextShapeRunProvider for EdgeShapeProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        _style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        _include_kerning: bool,
    ) -> crate::text::shaping::TextShapingOutcome {
        self.shape_calls = self.shape_calls.saturating_add(1);
        self.max_shaped_graphemes = self.max_shaped_graphemes.max(text.graphemes(true).count());
        crate::text::shaping::TextShapingOutcome::Ready(Arc::new(shaped_run(
            text,
            direction,
            source_range,
        )))
    }
}

fn shaped_run(text: &str, direction: TextDirection, source_range: TextRange) -> ShapedGlyphRun {
    let mut glyphs = Vec::new();
    let mut measured_width = 0.0_f32;
    for (index, (start, grapheme)) in text.grapheme_indices(true).enumerate() {
        let mut advance = if grapheme == "-" { 4.0 } else { 10.0 };
        if index == 0 && grapheme == "A" {
            advance += 1.0;
        }
        if grapheme == "Z" {
            advance += 2.0;
        }
        if grapheme == "V" && text.ends_with("V-") {
            advance -= 2.0;
        }
        let end = start + grapheme.len();
        glyphs.push(test_glyph(start, end, advance, direction));
        measured_width += advance;
    }
    ShapedGlyphRun {
        source_text: std::sync::Arc::from(text),
        source_range,
        unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
        primary_face_id: None,
        direction,
        orientation: TextOrientation::Horizontal,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width,
        measured_height: 12.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
            line_index: 0,
            source_range,
            visual_range: source_range,
            measured_width,
            baseline: 9.0,
            line_height: 12.0,
            glyphs,
        }],
    }
}

fn test_glyph(
    source_start: usize,
    source_end: usize,
    advance: f32,
    direction: TextDirection,
) -> ShapedGlyph {
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
        direction,
        bidi_level: 0,
        cluster_flags: ShapedGlyphClusterFlags::default(),
        rotation: ShapedGlyphRotation::None,
        script: ShapedGlyphScript::default(),
    }
}
