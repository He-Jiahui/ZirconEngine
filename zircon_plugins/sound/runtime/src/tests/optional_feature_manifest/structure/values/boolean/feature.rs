use super::super::super::sources::*;

#[test]
fn optional_feature_boolean_feature_projection_calls_raw_parser_owner() {
    assert!(
        BOOLEAN_FEATURE.contains("super::raw::bool_from_plugin_toml(value)"),
        "feature boolean projection should call the raw parser from its child owner"
    );
}
