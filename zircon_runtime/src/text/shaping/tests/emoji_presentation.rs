use super::cluster_uses_emoji_presentation;

#[test]
fn presentation_follows_unicode_properties_and_variation_selectors() {
    assert!(cluster_uses_emoji_presentation("\u{1f600}"));
    assert!(!cluster_uses_emoji_presentation("\u{2600}"));
    assert!(cluster_uses_emoji_presentation("\u{2600}\u{fe0f}"));
    assert!(!cluster_uses_emoji_presentation("\u{1f600}\u{fe0e}"));
    assert!(!cluster_uses_emoji_presentation("\u{1f02c}"));
    assert!(!cluster_uses_emoji_presentation("A\u{fe0f}"));
}

#[test]
fn keycaps_support_both_standard_selector_forms() {
    assert!(cluster_uses_emoji_presentation("1\u{20e3}"));
    assert!(cluster_uses_emoji_presentation("1\u{fe0f}\u{20e3}"));
}
