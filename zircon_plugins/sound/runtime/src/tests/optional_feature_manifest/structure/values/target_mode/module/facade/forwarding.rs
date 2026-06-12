use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_module_facade_does_not_own_forwarding_body() {
    assert!(
        !TARGET_MODE_MODULE.contains("fn module_target_mode_list_from_plugin_toml")
            && !TARGET_MODE_MODULE
                .contains("super::list::runtime_target_mode_list_from_plugin_toml(value)"),
        "module target-mode projection parent must not own semantic forwarding bodies"
    );
}
