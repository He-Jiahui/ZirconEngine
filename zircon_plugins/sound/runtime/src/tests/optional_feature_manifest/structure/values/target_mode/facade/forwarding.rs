use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_facade_does_not_own_module_forwarding_body() {
    assert!(
        !TARGET_MODE_ROOT.contains("fn module_target_mode_list_from_plugin_toml"),
        "target_mode parent must not own semantic module target-mode forwarding bodies"
    );
}
