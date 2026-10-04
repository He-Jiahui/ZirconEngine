use super::normalized_tag;

#[test]
fn normalized_tag_trims_and_folds_valid_ascii() {
    assert_eq!(
        normalized_tag("  Color_Accent  ").as_deref(),
        Some("color_accent")
    );
}

#[test]
fn normalized_tag_rejects_invalid_or_non_ascii_names() {
    for tag in ["", "bad-tag", "bad tag", "café"] {
        assert_eq!(normalized_tag(tag), None);
    }
}
