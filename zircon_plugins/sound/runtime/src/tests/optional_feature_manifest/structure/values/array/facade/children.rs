use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_facade_declares_value_children() {
    assert!(
        ARRAY_ROOT.contains("mod feature;")
            && ARRAY_ROOT.contains("mod list;")
            && ARRAY_ROOT.contains("mod module;")
            && ARRAY_ROOT.contains("mod raw;"),
        "array parent must remain a structural child-module owner"
    );
}
