use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_defaults_entry_calls_semantic_enum_list_child() {
    assert!(
        PACKAGING_DEFAULTS_ENTRY
            .contains("super::super::list::packaging_strategy_list_from_plugin_toml(value)"),
        "default packaging projection entry should call the semantic enum-list child"
    );
}
