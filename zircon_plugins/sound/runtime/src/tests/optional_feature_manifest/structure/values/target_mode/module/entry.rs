use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_module_entry_calls_semantic_enum_list_child() {
    assert!(
        TARGET_MODE_MODULE_ENTRY
            .contains("super::super::list::runtime_target_mode_list_from_plugin_toml(value)"),
        "module target-mode projection entry should call the semantic enum-list child"
    );
}
