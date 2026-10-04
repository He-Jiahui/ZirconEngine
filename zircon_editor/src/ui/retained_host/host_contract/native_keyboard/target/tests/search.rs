use super::*;

#[test]
fn popup_prefix_matching_preserves_unicode_lowercase_semantics() {
    let value: SharedString = "İstanbul".into();

    assert!(popup_text_starts_with(&value, "i\u{307}s"));
    assert!(!popup_text_starts_with(&value, "is"));
    assert!(popup_text_starts_with(&value, ""));
}
