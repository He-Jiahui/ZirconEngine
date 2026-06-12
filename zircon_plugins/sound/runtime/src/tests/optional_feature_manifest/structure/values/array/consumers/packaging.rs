use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_packaging_consumer_uses_semantic_list_helper() {
    assert!(
        PACKAGING_LIST.contains("super::super::array::string_list_from_plugin_toml(value)"),
        "packaging enum-list projection should consume the semantic string-list helper"
    );
}
