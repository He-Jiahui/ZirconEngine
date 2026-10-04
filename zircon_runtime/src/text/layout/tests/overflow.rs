use super::{retained_grapheme_counts, EllipsisPlacement};
use unicode_segmentation::UnicodeSegmentation;

#[test]
fn end_word_ellipsis_drops_an_incomplete_first_word() {
    let text = "alpha beta";
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    let advances = vec![1.0; graphemes.len()];

    assert_eq!(
        retained_grapheme_counts(text, &graphemes, &advances, 8.0, EllipsisPlacement::EndWord),
        (5, 0),
        "the incomplete `be` prefix must not survive a word ellipsis"
    );
}

#[test]
fn end_word_ellipsis_uses_unicode_boundaries_without_whitespace() {
    let text = "alpha-beta";
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    let advances = vec![1.0; graphemes.len()];

    assert_eq!(
        retained_grapheme_counts(text, &graphemes, &advances, 8.0, EllipsisPlacement::EndWord),
        (5, 0),
        "the completed UAX #29 word survives even when the separator is punctuation"
    );
}

#[test]
fn end_word_ellipsis_supports_cjk_without_spaces() {
    let text = "中文文本";
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    let advances = vec![1.0; graphemes.len()];

    assert_eq!(
        retained_grapheme_counts(text, &graphemes, &advances, 3.0, EllipsisPlacement::EndWord),
        (3, 0)
    );
}

#[test]
fn middle_ellipsis_preserves_the_existing_suffix_first_selection_order() {
    let text = "abcd";
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();

    assert_eq!(
        retained_grapheme_counts(
            text,
            &graphemes,
            &[1.0, 5.0, 5.0, 1.0],
            6.0,
            EllipsisPlacement::Middle,
        ),
        (1, 1),
        "middle ellipsis must keep the old suffix-first alternating selection policy"
    );
}
