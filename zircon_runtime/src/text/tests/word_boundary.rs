use super::*;

#[test]
fn map_retains_the_unicode_snapshot_identity() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);

    assert_eq!(
        WordBoundaryMap::for_snapshot("text", next).unicode_data_snapshot(),
        next
    );
}

#[test]
fn completed_prefix_uses_unicode_words_instead_of_whitespace_tokens() {
    let hyphenated = WordBoundaryMap::new("alpha-beta");
    assert_eq!(hyphenated.completed_prefix_end(8), 5);

    let apostrophe = WordBoundaryMap::new("go can't");
    assert_eq!(apostrophe.completed_prefix_end(6), 2);
    assert_eq!(apostrophe.completed_prefix_end(8), 8);
}

#[test]
fn completed_prefix_supports_text_without_spaces() {
    let text = "中文文本";
    let third_ideograph_end = "中文文".len();

    assert_eq!(
        WordBoundaryMap::new(text).completed_prefix_end(third_ideograph_end),
        third_ideograph_end
    );
}

#[test]
fn navigation_queries_share_the_same_word_ranges() {
    let map = WordBoundaryMap::new("alpha-beta");

    assert_eq!(map.previous_word_start(8), Some(6));
    assert_eq!(map.previous_word_start(6), Some(0));
    assert_eq!(map.next_word_end(5), Some(10));
    assert_eq!(map.word_range_at(7), Some(TextRange { start: 6, end: 10 }));
}
