use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_list_consumes_semantic_array_helper() {
    assert!(
        PACKAGING_LIST.contains("super::super::array::string_list_from_plugin_toml(value)"),
        "packaging enum-list projection should consume the semantic string-list helper"
    );
}
