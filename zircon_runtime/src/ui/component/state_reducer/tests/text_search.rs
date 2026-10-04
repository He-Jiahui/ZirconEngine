use super::{contains_lowercase_query, starts_with_lowercase_query};

#[test]
fn ascii_search_is_case_insensitive_and_observes_existing_trim_rules() {
    assert!(starts_with_lowercase_query("  Open Project", "open"));
    assert!(contains_lowercase_query("  Editor.SaveAll  ", "save"));
    assert!(!starts_with_lowercase_query("Reopen Project", "open"));
    assert!(!contains_lowercase_query("Editor.SaveAll", "close"));
}

#[test]
fn unicode_search_preserves_lowercase_matching_behavior() {
    assert!(starts_with_lowercase_query("  \u{c5}ngstrom", "\u{e5}ng"));
    assert!(contains_lowercase_query(
        "  \u{c9}DITEUR DE SCENE  ",
        "\u{e9}diteur"
    ));
}

#[test]
fn empty_query_matches_without_slicing_an_empty_ascii_window() {
    assert!(starts_with_lowercase_query("Open", ""));
    assert!(contains_lowercase_query("Open", ""));
}
