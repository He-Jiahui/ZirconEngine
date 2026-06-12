use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_module_facade_declares_field_children() {
    assert!(
        STRING_MODULE.contains("mod crate_name;") && STRING_MODULE.contains("mod name;"),
        "module string domain should declare field-owner children"
    );
}
