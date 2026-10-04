use std::collections::HashSet;

use super::TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES;

#[test]
fn grapheme_index_profile_uses_only_fixed_names() {
    let unique = TEXT_DOCUMENT_GRAPHEME_PROFILE_COUNTER_NAMES
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(unique.len(), 12);
    assert!(unique
        .iter()
        .all(|name| name.starts_with("text_document_grapheme_")));
}
