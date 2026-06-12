use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_module_facade_declares_entry_child() {
    assert!(
        TARGET_MODE_MODULE.contains("mod entry;"),
        "module target-mode projection parent should declare the entry owner"
    );
}
