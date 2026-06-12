use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_feature_capability_projection_uses_semantic_list_owner() {
    assert!(
        ARRAY_FEATURE.contains("super::list::string_list_from_plugin_toml(value)"),
        "feature capability projection should call the semantic string-list child"
    );
}
