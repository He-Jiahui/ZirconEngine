use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_facade_declares_value_children() {
    assert!(
        TARGET_MODE_ROOT.contains("mod list;")
            && TARGET_MODE_ROOT.contains("mod module;")
            && TARGET_MODE_ROOT.contains("mod raw;"),
        "target_mode parent must remain a structural child-module owner"
    );
}
