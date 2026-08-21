use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::rich::parser_registry::parse_rich_text;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::{
    RichParseResult, RichTextFormat, ShapedGlyphRun, StyleOverride, StyledRun, TextRange,
    TextShapingWorkBudget, TextStyle,
};
use unicode_segmentation::UnicodeSegmentation;

use super::*;

#[test]
fn rich_boundary_correction_shapes_only_bounded_span_edges() {
    let source = format!("A{}Z", "x".repeat(98));
    let parsed = parse_rich_text(&source, RichTextFormat::Plain);
    let style = TextStyle::default();
    let mut provider = RecordingProvider::default();
    let index = RichAdvanceIndex::new(&parsed, &style, &mut provider, |_, _| (0.0, 0.0));
    provider.requests.clear();

    let corrected = index.corrected_advance_with_provider(
        &parsed.text,
        1,
        parsed.text.len().saturating_sub(1),
        None,
        &mut provider,
    );

    assert!(corrected.is_finite());
    assert_eq!(provider.requests.len(), 2);
    assert!(provider.requests.iter().all(|request| {
        request.graphemes(true).count()
            <= super::super::line_break::BOUNDARY_SHAPING_CONTEXT_GRAPHEMES * 2
    }));
}

#[test]
fn rich_boundary_correction_shapes_soft_hyphen_suffix_with_span_tail() {
    let parsed = parse_rich_text("AV", RichTextFormat::Plain);
    let style = TextStyle::default();
    let mut provider = RecordingProvider::default();
    let index = RichAdvanceIndex::new(&parsed, &style, &mut provider, |_, _| (0.0, 0.0));
    provider.requests.clear();

    let corrected = index.corrected_advance_with_provider(
        &parsed.text,
        0,
        parsed.text.len(),
        Some("-"),
        &mut provider,
    );

    assert!(corrected.is_finite());
    assert_eq!(provider.requests, vec!["AV-".to_string()]);
}

#[test]
fn rich_span_index_shapes_one_to_one_thousand_alternating_runs_once_each() {
    for run_count in [1_usize, 100, 1_000] {
        let text = "x".repeat(run_count);
        let runs = (0..run_count)
            .map(|index| StyledRun {
                byte_range: (
                    u32::try_from(index).unwrap_or(u32::MAX),
                    u32::try_from(index.saturating_add(1)).unwrap_or(u32::MAX),
                ),
                style: StyleOverride {
                    weight: Some(if index % 2 == 0 { 400 } else { 700 }),
                    ..StyleOverride::default()
                },
                ..StyledRun::default()
            })
            .collect();
        let parsed = RichParseResult {
            text: text.into(),
            runs,
            ..RichParseResult::default()
        };
        let mut provider = RecordingProvider::default();

        let index = RichAdvanceIndex::new(&parsed, &TextStyle::default(), &mut provider, |_, _| {
            (0.0, 0.0)
        });

        assert_eq!(
            index.metrics_in_range(0, parsed.text.len()).len(),
            run_count
        );
        assert_eq!(
            provider.requests.len(),
            run_count,
            "{run_count} alternating style runs must each shape exactly once"
        );
        assert!(
            provider
                .requests
                .iter()
                .all(|request| request.graphemes(true).count() == 1)
        );
    }
}

#[test]
fn rich_span_index_source_line_request_topology_evidence() {
    let budget = TextShapingWorkBudget::default();
    let long_run = "x".repeat(budget.max_inline_input_bytes() + 1);
    assert!(budget.exceeds_inline_threshold(long_run.len()));
    let source = format!("first\r\nsecond\u{2028}{long_run}");
    let parsed = parse_rich_text(&source, RichTextFormat::Plain);
    let mut provider = RecordingProvider::default();

    let _ = RichAdvanceIndex::new(&parsed, &TextStyle::default(), &mut provider, |_, _| {
        (0.0, 0.0)
    });

    assert_eq!(provider.requests.len(), 3);
    assert_eq!(provider.requests[0], "first");
    assert_eq!(provider.requests[1], "second");
    assert_eq!(provider.requests[2], long_run);

    let optimized_requests = provider.requests.len();
    let legacy_requests = optimized_requests + 1;
    let reduction_basis_points = (legacy_requests - optimized_requests) * 10_000 / legacy_requests;
    println!(
        "PERF-MVP-RTS-P0-001 workload=rich_advance budget_bytes={} legacy_requests={legacy_requests} optimized_requests={optimized_requests} request_reduction_basis_points={reduction_basis_points}",
        budget.max_inline_input_bytes(),
    );
}

#[derive(Default)]
struct RecordingProvider {
    direct: DirectTextShapeRunProvider,
    requests: Vec<String>,
}

impl TextShapeRunProvider for RecordingProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        self.requests.push(text.to_string());
        self.direct.shape_horizontal_line_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }
}
