/// Shared domain filtering for the existing SearchInput/ListRow projection.
/// Uses borrowed UTF-8 substrings and ASCII case folding without allocating.
pub(super) fn matches_query(query: &str, text: &str, value: &str) -> bool {
    query.split_whitespace().all(|word| {
        [text, value].iter().any(|field| {
            field
                .as_bytes()
                .windows(word.len())
                .any(|part| part.eq_ignore_ascii_case(word.as_bytes()))
        })
    })
}
