use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_dependency_plugin_id_field_calls_raw_parser() {
    assert!(
        STRING_DEPENDENCY_PLUGIN_ID.contains("super::super::raw::string_from_plugin_toml(value)"),
        "dependency plugin-id string field projection should call the raw parser from its child owner"
    );
}
