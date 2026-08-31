use crate::core::framework::text::TextLayoutError;
use crate::text::RichTextFormat;
use crate::text::SharedTextLayoutSession;
use crate::text::layout::GraphemeAdvanceMetric;
use crate::ui::text::rich_text::{UiParsedText, parse_source_text as try_parse_source_text};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRange, UiTextWrap};

use super::*;

fn parse_source_text(text: &str, format: RichTextFormat) -> UiParsedText {
    try_parse_source_text(text, format).expect("test text fits parser budgets")
}

#[test]
fn glyph_wrap_shapes_complete_source_and_bounded_line_edges() {
    let parsed = parse_source_text(
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMN",
        RichTextFormat::Plain,
    );
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let mut provider = SharedTextLayoutSession::new();
    let before = provider.cache_report();

    let lines = wrap_source_runs_with_provider(
        &parsed.runs,
        UiTextWrap::Glyph,
        12.0,
        &style,
        &mut provider,
    );
    let after = provider.cache_report();

    assert!(lines.len() > 1, "the fixture must exercise glyph wrapping");
    let misses = after.miss_count.saturating_sub(before.miss_count);
    assert!(
        misses > 1,
        "glyph wrap must correct the tentative line boundaries"
    );
    assert!(
        misses
            <= 1_u64.saturating_add(
                u64::try_from(lines.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(
                        u64::try_from(
                            2 * crate::text::layout::BOUNDARY_SHAPING_CONTEXT_GRAPHEMES + 1,
                        )
                        .unwrap_or(u64::MAX),
                    ),
            ),
        "boundary correction misses must remain linear in the produced line count"
    );
}

#[test]
fn wrapping_source_has_no_production_growing_prefix_candidate() {
    let source = include_str!("../wrapping.rs");

    assert!(!source.contains("should_wrap_before_chunk_with_provider"));
    assert!(!source.contains("candidate.push_str(current_text)"));
    assert!(!source.contains("Vec<TextSegment"));
    assert!(source.contains("GraphemeAdvanceIndex::measured_with_provider"));
    assert!(source.contains("corrected_glyph_ranges_with_provider"));
    assert!(source.contains("append_corrected_glyph_ranges"));
    assert!(source.contains("visit_source_segments_preserving_hard_lines"));
}

#[test]
fn corrected_glyph_ranges_are_committed_without_rechecking_each_grapheme() {
    let style = UiResolvedStyle::default();
    let neutral_style = text_style(&style);
    let mut provider = SharedTextLayoutSession::new();
    let index = GraphemeAdvanceIndex::measured_with_provider("abcd", &neutral_style, &mut provider)
        .into_result()
        .expect("measure glyph range fixture");
    let mut lines = Vec::new();
    let mut current = CandidateLine::empty();
    append_segment(
        &mut current,
        zircon_runtime_interface::ui::surface::UiTextRunKind::Plain,
        "p",
        UiTextRange { start: 0, end: 1 },
    );
    let mut current_advance = 3.0;

    append_corrected_glyph_ranges(
        &mut lines,
        &mut current,
        zircon_runtime_interface::ui::surface::UiTextRunKind::Plain,
        "abcd",
        UiTextRange { start: 1, end: 5 },
        &index,
        vec![(0, 2), (2, 4)],
        &mut current_advance,
    )
    .into_result()
    .expect("corrected glyph ranges remain valid");

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, "pab");
    assert_eq!(current.text, "cd");
    assert!((current_advance - index.advance(2, 4)).abs() < 0.01);
}

#[test]
fn corrected_glyph_ranges_fail_closed_when_a_metric_cannot_be_sliced() {
    let index = GraphemeAdvanceIndex::from_metrics(vec![GraphemeAdvanceMetric {
        source_start: 1,
        source_end: 9,
        advance: 4.0,
        cross_extent: 10.0,
    }]);
    let mut lines = Vec::new();
    let mut current = CandidateLine::empty();
    let mut current_advance = 0.0;

    let outcome = append_corrected_glyph_ranges(
        &mut lines,
        &mut current,
        zircon_runtime_interface::ui::surface::UiTextRunKind::Plain,
        "ab",
        UiTextRange { start: 0, end: 2 },
        &index,
        vec![(0, 1)],
        &mut current_advance,
    );

    assert_eq!(outcome.into_result(), Err(TextLayoutError::LayoutFailed));
    assert!(lines.is_empty());
    assert!(current.text.is_empty());
}

#[test]
fn source_segmentation_preserves_all_mandatory_unicode_breaks() {
    let mut segments = Vec::new();
    visit_source_segments_preserving_hard_lines("a\r\nb\u{2028}c", 10, |segment| {
        segments.push((
            segment.text.to_owned(),
            segment.range.start,
            segment.range.end,
            segment.hard_break,
        ));
    });

    assert_eq!(
        segments,
        vec![
            ("a".to_owned(), 10, 11, false),
            ("\r\n".to_owned(), 11, 13, true),
            ("b".to_owned(), 13, 14, false),
            ("\u{2028}".to_owned(), 14, 17, true),
            ("c".to_owned(), 17, 18, false),
        ]
    );
}

#[test]
fn source_segment_visitor_emits_an_empty_segment_for_empty_source() {
    let mut segments = Vec::new();
    visit_source_segments_preserving_hard_lines("", 24, |segment| {
        segments.push((
            segment.text.to_owned(),
            segment.range.start,
            segment.range.end,
            segment.hard_break,
        ));
    });

    assert_eq!(segments, vec![("".to_owned(), 24, 24, false)]);
}

#[test]
fn source_segment_visitor_does_not_publish_a_shaping_budget_boundary() {
    let budget = crate::text::TextShapingWorkBudget::default();
    let text = "a".repeat(budget.max_inline_input_bytes() + 1);
    assert!(budget.exceeds_inline_threshold(text.len()));
    let mut segments = Vec::new();
    visit_source_segments_preserving_hard_lines(&text, 0, |segment| {
        segments.push((segment.text.len(), segment.hard_break));
    });

    assert_eq!(segments, vec![(text.len(), false)]);
}

#[test]
fn rich_runs_joined_without_wrap_remain_one_logical_candidate_line() {
    let budget = crate::text::TextShapingWorkBudget::default();
    let boundary = budget.max_inline_input_bytes();
    let first = "x".repeat(boundary / 2);
    let second = "x".repeat(boundary / 2 + 1);
    let source = format!("{first}<b>{second}</b>");
    let parsed = parse_source_text(&source, RichTextFormat::HtmlSubsetV1);
    let style = UiResolvedStyle::default();
    let mut provider = SharedTextLayoutSession::new();

    let lines = wrap_source_runs_with_provider(
        &parsed.runs,
        UiTextWrap::None,
        f32::MAX,
        &style,
        &mut provider,
    );

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text.len(), boundary + 1);
    assert_eq!(
        lines[0].source_range,
        UiTextRange {
            start: 0,
            end: boundary + 1,
        }
    );
}

#[test]
fn word_wrap_boundary_cache_misses_remain_linear_at_scale() {
    for token_count in [1_usize, 100, 1_000] {
        let source = (0..token_count)
            .map(|index| format!("w{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let parsed = parse_source_text(&source, RichTextFormat::Plain);
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let mut provider = SharedTextLayoutSession::new();
        let before = provider.cache_report();

        let lines = wrap_source_runs_with_provider(
            &parsed.runs,
            UiTextWrap::Word,
            80.0,
            &style,
            &mut provider,
        );
        let misses = provider
            .cache_report()
            .miss_count
            .saturating_sub(before.miss_count);
        let grapheme_count = source.graphemes(true).count();

        assert!(!lines.is_empty());
        assert!(
            misses
                <= u64::try_from(grapheme_count)
                    .unwrap_or(u64::MAX)
                    .saturating_mul(4)
                    .saturating_add(4),
            "{token_count} tokens exceeded the linear shaped-cache miss budget"
        );
    }
}
