use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_owner_plugin_field_calls_raw_parser() {
    assert!(
        STRING_FEATURE_OWNER_PLUGIN.contains("super::super::raw::string_from_plugin_toml(value)"),
        "feature owner-plugin string field projection should call the raw parser from its child owner"
    );
}
