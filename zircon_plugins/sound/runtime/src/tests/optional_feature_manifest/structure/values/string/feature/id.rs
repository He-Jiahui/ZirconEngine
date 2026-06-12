use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_id_field_calls_raw_parser() {
    assert!(
        STRING_FEATURE_ID.contains("super::super::raw::string_from_plugin_toml(value)"),
        "feature id string field projection should call the raw parser from its child owner"
    );
}
