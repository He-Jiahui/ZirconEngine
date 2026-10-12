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
#[path = "tests/search.rs"]
mod tests;
