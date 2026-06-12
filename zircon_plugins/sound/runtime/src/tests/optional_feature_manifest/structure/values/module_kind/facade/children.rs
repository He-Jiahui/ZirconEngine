use super::super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_facade_declares_value_children() {
    assert!(
        MODULE_KIND_ROOT.contains("mod module;") && MODULE_KIND_ROOT.contains("mod raw;"),
        "module_kind parent must remain a structural child-module owner"
    );
}
