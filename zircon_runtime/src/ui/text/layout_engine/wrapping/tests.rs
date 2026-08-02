use crate::text::RichTextFormat;
use crate::text::SharedTextLayoutSession;
use crate::ui::text::rich_text::parse_source_text;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextWrap};

use super::*;

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
    assert!(source.contains("GraphemeAdvanceIndex::measured_with_provider"));
    assert!(source.contains("corrected_glyph_ranges_with_provider"));
}

#[test]
fn source_segmentation_preserves_all_mandatory_unicode_breaks() {
    let segments = split_preserving_hard_lines("a\r\nb\u{2028}c", 10);

    assert_eq!(
        segments
            .iter()
            .map(|segment| (
                segment.text,
                segment.range.start,
                segment.range.end,
                segment.hard_break
            ))
            .collect::<Vec<_>>(),
        vec![
            ("a", 10, 11, false),
            ("\r\n", 11, 13, true),
            ("b", 13, 14, false),
            ("\u{2028}", 14, 17, true),
            ("c", 17, 18, false),
        ]
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
