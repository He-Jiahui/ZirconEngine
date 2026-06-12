use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_display_name_field_calls_raw_parser() {
    assert!(
        STRING_FEATURE_DISPLAY_NAME.contains("super::super::raw::string_from_plugin_toml(value)"),
        "feature display-name string field projection should call the raw parser from its child owner"
    );
}
