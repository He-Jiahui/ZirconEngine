use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_defaults_facade_declares_entry_child() {
    assert!(
        PACKAGING_DEFAULTS.contains("mod entry;"),
        "default packaging projection parent should declare the entry owner"
    );
}
