use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_target_mode_consumer_uses_semantic_list_helper() {
    assert!(
        TARGET_MODE_LIST.contains("super::super::array::string_list_from_plugin_toml(value)"),
        "target-mode enum-list projection should consume the semantic string-list helper"
    );
}
