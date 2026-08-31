use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) fn normalized_popup_text_query(
    text: &str,
) -> Option<String> {
    let query = text.trim().to_lowercase();
    if query.is_empty() {
        None
    } else {
        Some(query)
    }
}

pub(in crate::ui::retained_host::host_contract) fn popup_text_starts_with(
    value: &SharedString,
    query: &str,
) -> bool {
    let mut value_lowercase = value.as_str().chars().flat_map(char::to_lowercase);
    query
        .chars()
        .all(|expected| value_lowercase.next() == Some(expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_prefix_matching_preserves_unicode_lowercase_semantics() {
        let value: SharedString = "İstanbul".into();

        assert!(popup_text_starts_with(&value, "i\u{307}s"));
        assert!(!popup_text_starts_with(&value, "is"));
        assert!(popup_text_starts_with(&value, ""));
    }
}
