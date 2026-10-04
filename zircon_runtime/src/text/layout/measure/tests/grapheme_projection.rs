use super::{aligned_grapheme_span, GraphemeProjectionCursor};
use crate::text::TextRange;

#[test]
fn cursor_advances_monotonic_ranges_and_falls_back_for_reordered_ranges() {
    let graphemes = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
    let mut cursor = GraphemeProjectionCursor::default();

    assert_eq!(cursor.overlap_bounds(&graphemes, 0, 1), (0, 1));
    assert_eq!(cursor.overlap_bounds(&graphemes, 1, 3), (1, 3));
    assert_eq!(cursor.overlap_bounds(&graphemes, 3, 5), (3, 5));

    // Visual RTL/reordered clusters must remain exact even after the fast path was used.
    assert_eq!(cursor.overlap_bounds(&graphemes, 1, 2), (1, 2));
    assert_eq!(cursor.overlap_bounds(&graphemes, 2, 4), (2, 4));
}

#[test]
fn aligned_grapheme_span_rejects_partial_boundaries() {
    let graphemes = [(0, 1), (1, 3), (3, 4)];

    assert_eq!(
        aligned_grapheme_span(&graphemes, 1, 3, TextRange { start: 1, end: 4 }),
        Some(2.0)
    );
    assert_eq!(
        aligned_grapheme_span(&graphemes, 1, 3, TextRange { start: 2, end: 4 }),
        None
    );
}
