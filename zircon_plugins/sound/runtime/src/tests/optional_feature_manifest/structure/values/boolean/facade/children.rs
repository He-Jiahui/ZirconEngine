use super::super::super::super::sources::*;

#[test]
fn optional_feature_boolean_facade_declares_value_children() {
    assert!(
        BOOLEAN_ROOT.contains("mod dependency;")
            && BOOLEAN_ROOT.contains("mod feature;")
            && BOOLEAN_ROOT.contains("mod raw;"),
        "boolean parent must remain a structural child-module owner"
    );
}
