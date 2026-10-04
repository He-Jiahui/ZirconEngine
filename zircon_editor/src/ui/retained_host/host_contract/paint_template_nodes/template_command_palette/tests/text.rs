use super::*;

#[test]
fn command_palette_search_text_uses_placeholder_only_for_empty_queries() {
    let empty = command_palette_search_text("   ");
    assert_eq!(empty.value, SEARCH_PLACEHOLDER);
    assert!(empty.placeholder);

    let query = command_palette_search_text("lights");
    assert_eq!(query.value, "lights");
    assert!(!query.placeholder);
}
