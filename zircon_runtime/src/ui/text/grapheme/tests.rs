use super::{
    clamp_grapheme_boundary, line_end_boundary, line_start_boundary,
    next_line_same_column_boundary, next_word_boundary, previous_line_same_column_boundary,
    previous_word_boundary, word_range_at,
};

#[test]
fn grapheme_boundary_floor_rejects_combining_and_zwj_interior_offsets() {
    let combining = "a\u{0301}b";
    assert_eq!(clamp_grapheme_boundary(combining, 1), 0);
    assert_eq!(clamp_grapheme_boundary(combining, 2), 0);
    assert_eq!(clamp_grapheme_boundary(combining, 3), 3);

    let zwj = "\u{1f469}\u{200d}\u{1f680}x";
    assert_eq!(clamp_grapheme_boundary(zwj, 4), 0);
    assert_eq!(clamp_grapheme_boundary(zwj, 7), 0);
    assert_eq!(clamp_grapheme_boundary(zwj, 11), 11);
}

#[test]
fn line_navigation_uses_every_canonical_hard_separator() {
    let text = "ab\u{2028}cd\u{0085}ef\u{2029}gh";
    let second_start = "ab\u{2028}".len();
    let third_start = "ab\u{2028}cd\u{0085}".len();
    let fourth_start = "ab\u{2028}cd\u{0085}ef\u{2029}".len();

    assert_eq!(line_start_boundary(text, third_start + 1), third_start);
    assert_eq!(
        line_end_boundary(text, second_start),
        third_start - "\u{0085}".len()
    );
    assert_eq!(
        previous_line_same_column_boundary(text, third_start + 1),
        Some(second_start + 1)
    );
    assert_eq!(
        next_line_same_column_boundary(text, third_start + 1),
        Some(fourth_start + 1)
    );
}

#[test]
fn line_navigation_keeps_crlf_as_one_separator() {
    let text = "ab\r\ncd\u{000b}ef\u{000c}gh";
    let second_start = "ab\r\n".len();
    let third_start = "ab\r\ncd\u{000b}".len();
    let fourth_start = "ab\r\ncd\u{000b}ef\u{000c}".len();

    assert_eq!(line_start_boundary(text, third_start + 1), third_start);
    assert_eq!(line_end_boundary(text, second_start), third_start - 1);
    assert_eq!(line_end_boundary(text, second_start - 1), second_start - 2);
    assert_eq!(
        previous_line_same_column_boundary(text, second_start + 1),
        Some(1)
    );
    assert_eq!(
        previous_line_same_column_boundary(text, third_start + 1),
        Some(second_start + 1)
    );
    assert_eq!(
        next_line_same_column_boundary(text, third_start + 1),
        Some(fourth_start + 1)
    );
}

#[test]
fn word_navigation_consumes_the_shared_unicode_boundary_owner() {
    let text = "alpha-beta can't";

    assert_eq!(previous_word_boundary(text, 8), Some(6));
    assert_eq!(next_word_boundary(text, 5), Some(10));
    assert_eq!(word_range_at(text, 12), Some((11, text.len())));
}
