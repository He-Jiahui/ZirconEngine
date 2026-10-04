use crate::text::layout::TextLineMetrics;
use crate::text::TextRange;
use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

use super::super::candidate_line::append_segment;
use super::{
    fragment_input, maximum_line_height, raw_fragment_advances_are_layout_safe,
    source_congruent_range, total_line_height, CandidateLine,
};

#[test]
fn source_congruent_candidate_preserves_the_absolute_shaper_range() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "world",
        UiTextRange { start: 0, end: 5 },
    );

    assert_eq!(
        source_congruent_range(&line, "world", 11),
        Some(TextRange { start: 11, end: 16 })
    );
}

#[test]
fn tab_containing_source_congruent_candidate_retains_a_metric_fragment_input() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "alpha\tbeta",
        UiTextRange { start: 0, end: 10 },
    );

    let input = fragment_input(&line, "alpha\tbeta", 0)
        .expect("tab placement does not invalidate source-congruent font metrics");

    assert_eq!(input.source_range, TextRange { start: 0, end: 10 });
    assert_eq!(input.text, "alpha\tbeta");
    assert!(
        !raw_fragment_advances_are_layout_safe(&line),
        "tab stop placement must keep owning the final x advances"
    );
}

#[test]
fn source_congruent_range_rejects_an_unrepresentable_absolute_offset() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "word",
        UiTextRange { start: 0, end: 4 },
    );

    assert_eq!(source_congruent_range(&line, "word", usize::MAX), None);
}

#[test]
fn total_line_height_keeps_extreme_metric_accumulation_finite() {
    let metrics = [
        TextLineMetrics {
            width: 1.0,
            baseline: 1.0,
            line_height: f32::MAX,
        },
        TextLineMetrics {
            width: 1.0,
            baseline: 1.0,
            line_height: f32::MAX,
        },
    ];

    assert_eq!(total_line_height(&metrics), f32::MAX);
    assert!(total_line_height(&metrics).is_finite());
}

#[test]
fn maximum_line_height_publishes_only_finite_non_negative_geometry() {
    let metrics = [
        TextLineMetrics {
            width: 1.0,
            baseline: 1.0,
            line_height: f32::NAN,
        },
        TextLineMetrics {
            width: 1.0,
            baseline: 1.0,
            line_height: f32::INFINITY,
        },
    ];

    assert_eq!(maximum_line_height(&metrics, 12.0), f32::MAX);
    assert!(maximum_line_height(&metrics, 12.0).is_finite());
    assert_eq!(maximum_line_height(&[], f32::NAN), 0.0);
}
