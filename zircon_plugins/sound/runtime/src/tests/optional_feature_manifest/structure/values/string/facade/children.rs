use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_facade_declares_value_children() {
    assert!(
        STRING_ROOT.contains("mod dependency;")
            && STRING_ROOT.contains("mod feature;")
            && STRING_ROOT.contains("mod module;")
            && STRING_ROOT.contains("mod raw;"),
        "string parent must remain a structural child-module owner"
    );
}
