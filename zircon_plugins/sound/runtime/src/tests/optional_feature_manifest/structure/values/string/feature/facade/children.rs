use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_facade_declares_field_children() {
    assert!(
        STRING_FEATURE.contains("mod display_name;")
            && STRING_FEATURE.contains("mod id;")
            && STRING_FEATURE.contains("mod owner_plugin;"),
        "feature string domain should declare field-owner children"
    );
}
