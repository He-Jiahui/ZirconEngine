use crate::core::framework::text::TextDirection;
use crate::text::{compiled_unicode_data_snapshot_id, TextRange};
use unicode_segmentation::UnicodeSegmentation;

use super::{mirrored_bidi_char, BidiInvariantError, BidiParagraph};

#[test]
fn mirrors_unicode_bidi_pairs_only_on_odd_levels() {
    assert_eq!(mirrored_bidi_char('\u{27E8}', 1), Some('\u{27E9}'));
    assert_eq!(mirrored_bidi_char('\u{29C4}', 1), Some('\u{29C5}'));
    assert_eq!(mirrored_bidi_char('\u{27E8}', 2), None);
}

#[test]
fn bidi_line_order_rejects_glyph_ranges_outside_the_requested_line() {
    let bidi = BidiParagraph::new("abc", TextDirection::LeftToRight);

    assert_eq!(
        bidi.line_order(
            0..2,
            &[
                TextRange { start: 0, end: 1 },
                TextRange { start: 2, end: 3 }
            ],
        ),
        Err(BidiInvariantError::GlyphOutsideLine {
            glyph_index: 1,
            start: 2,
            end: 3,
            line_start: 0,
            line_end: 2,
        })
    );
}

#[test]
fn bidi_line_order_rejects_a_line_outside_the_source_paragraph() {
    let bidi = BidiParagraph::new("abc", TextDirection::LeftToRight);

    assert_eq!(
        bidi.line_order(0..4, &[TextRange { start: 0, end: 1 }]),
        Err(BidiInvariantError::InvalidLineRange { start: 0, end: 4 })
    );
}

#[test]
fn source_free_signature_replays_l1_for_a_wrapped_rtl_line() {
    let source = "\u{05D0}\u{05D1} abc ";
    let logical_ranges = source
        .grapheme_indices(true)
        .map(|(start, grapheme)| TextRange {
            start,
            end: start + grapheme.len(),
        })
        .collect::<Vec<_>>();
    let wrapped_end = logical_ranges[2].end;
    let bidi = BidiParagraph::new(source, TextDirection::Auto);
    let signature = bidi.line_signature(0..source.len()).unwrap();

    let expected = bidi
        .line_order(0..wrapped_end, &logical_ranges[..3])
        .unwrap();
    let actual = signature
        .line_order(0..wrapped_end, &logical_ranges[..3])
        .unwrap();

    // The third cluster is a trailing space in this physical line. This is the case that
    // cannot be recovered by slicing an already-reordered hard-line result.
    assert_eq!(actual, expected);
    assert_eq!(actual.resolved_base_direction, TextDirection::RightToLeft);
}

#[test]
fn source_free_signature_rejects_out_of_order_logical_glyph_ranges() {
    let bidi = BidiParagraph::new("abc", TextDirection::LeftToRight);
    let signature = bidi.line_signature(0..3).unwrap();

    assert_eq!(
        signature.line_order(
            0..3,
            &[
                TextRange { start: 1, end: 2 },
                TextRange { start: 0, end: 1 },
            ],
        ),
        Err(BidiInvariantError::NonMonotonicGlyphRange {
            glyph_index: 1,
            start: 0,
            previous_start: 1,
        })
    );
}

#[test]
fn source_free_signature_rejects_a_glyph_range_ending_inside_a_scalar() {
    let bidi = BidiParagraph::new("a\u{4E2D}", TextDirection::LeftToRight);
    let signature = bidi.line_signature(0..4).unwrap();

    assert_eq!(
        signature.line_order(0..4, &[TextRange { start: 0, end: 2 }]),
        Err(BidiInvariantError::MissingSignatureScalar { offset: 2 })
    );
}

#[test]
fn bidi_artifacts_retain_request_unicode_snapshot_identity() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);
    let paragraph = BidiParagraph::for_snapshot("abc", TextDirection::LeftToRight, next);
    let signature = paragraph
        .line_signature(0..3)
        .expect("valid paragraph signature");
    let order = signature
        .line_order(
            0..3,
            &[
                TextRange { start: 0, end: 1 },
                TextRange { start: 1, end: 3 },
            ],
        )
        .expect("valid line order");

    assert_eq!(paragraph.unicode_data_snapshot(), next);
    assert_eq!(signature.unicode_data_snapshot(), next);
    assert_eq!(order.unicode_data_snapshot, next);
}
