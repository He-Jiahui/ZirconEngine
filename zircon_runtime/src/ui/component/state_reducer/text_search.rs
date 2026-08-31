pub(super) fn starts_with_lowercase_query(value: &str, lowercase_query: &str) -> bool {
    let value = value.trim_start();
    if lowercase_query.is_empty() {
        return true;
    }
    if value.is_ascii() && lowercase_query.is_ascii() {
        return value
            .as_bytes()
            .get(..lowercase_query.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(lowercase_query.as_bytes()));
    }

    value.to_lowercase().starts_with(lowercase_query)
}

pub(super) fn contains_lowercase_query(value: &str, lowercase_query: &str) -> bool {
    let value = value.trim();
    if lowercase_query.is_empty() {
        return true;
    }
    if value.is_ascii() && lowercase_query.is_ascii() {
        return value
            .as_bytes()
            .windows(lowercase_query.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(lowercase_query.as_bytes()));
    }

    value.to_lowercase().contains(lowercase_query)
}

#[cfg(test)]
mod tests {
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
}
