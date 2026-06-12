use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_facade_declares_value_children() {
    assert!(
        PACKAGING_ROOT.contains("mod defaults;")
            && PACKAGING_ROOT.contains("mod list;")
            && PACKAGING_ROOT.contains("mod raw;"),
        "packaging parent must remain a structural child-module owner"
    );
}
