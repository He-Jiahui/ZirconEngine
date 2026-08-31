use crate::core::framework::text::TextDirection;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::{ShapedGlyphBreakSafety, TextRange};

use super::*;

#[test]
fn measured_advance_index_shapes_the_complete_source_once() {
    let mut provider = CountingShapeRunProvider::default();

    let index =
        GraphemeAdvanceIndex::measured_with_provider("abcd", &TextStyle::default(), &mut provider)
            .into_result()
            .expect("measure grapheme advance index");

    assert_eq!(provider.shape_calls, 1);
    assert_eq!(index.metrics_in_range(0, 4).len(), 4);
    assert!((index.advance(0, 2) + index.advance(2, 4) - index.advance(0, 4)).abs() < 0.01);
}

#[test]
fn atomic_cluster_moves_tentative_wrap_boundary_to_its_end() {
    let metrics = (0..4)
        .map(|source_start| GraphemeAdvanceMetric {
            source_start,
            source_end: source_start + 1,
            advance: 10.0,
            cross_extent: 10.0,
        })
        .collect();
    let index = GraphemeAdvanceIndex::from_metrics_and_clusters(
        metrics,
        vec![
            MeasuredGlyphCluster {
                source_range: TextRange { start: 0, end: 2 },
                advance: 20.0,
                caret_policy: MeasuredClusterCaretPolicy::AtomicCluster,
                break_safety: ShapedGlyphBreakSafety::RequiresReshape,
            },
            MeasuredGlyphCluster {
                source_range: TextRange { start: 2, end: 3 },
                advance: 10.0,
                caret_policy: MeasuredClusterCaretPolicy::GraphemeBoundary,
                break_safety: ShapedGlyphBreakSafety::Safe,
            },
            MeasuredGlyphCluster {
                source_range: TextRange { start: 3, end: 4 },
                advance: 10.0,
                caret_policy: MeasuredClusterCaretPolicy::GraphemeBoundary,
                break_safety: ShapedGlyphBreakSafety::Unknown,
            },
        ],
    );

    assert_eq!(
        index.coalesce_atomic_source_ranges(vec![(0, 1), (1, 3), (3, 4)]),
        vec![(0, 2), (2, 3), (3, 4)]
    );
    assert_eq!(
        index.break_safety_at_boundary(0),
        ShapedGlyphBreakSafety::Safe
    );
    assert_eq!(
        index.break_safety_at_boundary(1),
        ShapedGlyphBreakSafety::RequiresReshape
    );
    assert_eq!(
        index.break_safety_at_boundary(2),
        ShapedGlyphBreakSafety::Safe
    );
    assert_eq!(
        index.break_safety_at_boundary(3),
        ShapedGlyphBreakSafety::Unknown
    );
    assert_eq!(
        index.break_safety_at_boundary(4),
        ShapedGlyphBreakSafety::Safe
    );
    assert_eq!(
        index.break_safety_counts_at_monotonic_boundaries(0..=4),
        BoundaryBreakSafetyCounts {
            safe: 3,
            requires_reshape: 1,
            unknown: 1,
        }
    );
}

#[test]
fn atomic_cluster_geometry_exposes_only_legal_ltr_caret_edges() {
    let index = atomic_test_index();

    assert_eq!(index.ltr_atomic_caret_span(1), Some((0.0, 20.0)));
    assert_eq!(index.ltr_atomic_caret_span(0), None);
    assert_eq!(index.ltr_atomic_caret_span(2), None);
    assert_eq!(
        index.ltr_caret_hit(4.0),
        Some((TextRange { start: 0, end: 2 }, true))
    );
    assert_eq!(
        index.ltr_caret_hit(16.0),
        Some((TextRange { start: 0, end: 2 }, false))
    );
    assert_eq!(
        index.ltr_caret_hit(24.0),
        Some((TextRange { start: 2, end: 3 }, true))
    );
    assert_eq!(
        index.ltr_caret_hit(29.0),
        Some((TextRange { start: 2, end: 3 }, false))
    );
    assert_eq!(index.grapheme_boundary_index(0), 0);
    assert_eq!(index.grapheme_boundary_index(2), 2);
}

#[test]
fn atomic_cluster_geometry_expands_partial_selection_to_the_whole_cluster() {
    let index = atomic_test_index();

    assert_eq!(
        index.coalesce_atomic_source_range(TextRange { start: 0, end: 1 }),
        TextRange { start: 0, end: 2 }
    );
    assert_eq!(
        index.coalesce_atomic_source_range(TextRange { start: 1, end: 3 }),
        TextRange { start: 0, end: 3 }
    );
    assert_eq!(
        index.coalesce_atomic_source_range(TextRange { start: 2, end: 4 }),
        TextRange { start: 2, end: 4 }
    );
}

fn atomic_test_index() -> GraphemeAdvanceIndex {
    let metrics = (0..4)
        .map(|source_start| GraphemeAdvanceMetric {
            source_start,
            source_end: source_start + 1,
            advance: 10.0,
            cross_extent: 10.0,
        })
        .collect();
    GraphemeAdvanceIndex::from_metrics_and_clusters(
        metrics,
        vec![
            MeasuredGlyphCluster {
                source_range: TextRange { start: 0, end: 2 },
                advance: 20.0,
                caret_policy: MeasuredClusterCaretPolicy::AtomicCluster,
                break_safety: ShapedGlyphBreakSafety::RequiresReshape,
            },
            MeasuredGlyphCluster {
                source_range: TextRange { start: 2, end: 3 },
                advance: 10.0,
                caret_policy: MeasuredClusterCaretPolicy::GraphemeBoundary,
                break_safety: ShapedGlyphBreakSafety::Safe,
            },
            MeasuredGlyphCluster {
                source_range: TextRange { start: 3, end: 4 },
                advance: 10.0,
                caret_policy: MeasuredClusterCaretPolicy::GraphemeBoundary,
                break_safety: ShapedGlyphBreakSafety::Unknown,
            },
        ],
    )
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
