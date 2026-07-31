#[test]
fn named_layout_validation_uses_borrowed_static_metadata() {
    let source = include_str!("channel_layout.rs");

    assert!(source.contains("fn named_layout_contract("));
    assert!(!source.contains("Self::from_name(&self.name)"));
}

#[test]
fn speaker_uniqueness_validation_is_single_pass() {
    let source = include_str!("channel_layout.rs");

    assert!(source.contains("fn speaker_bit("));
    assert!(!source.contains("self.speakers[index + 1..].contains(speaker)"));
}
