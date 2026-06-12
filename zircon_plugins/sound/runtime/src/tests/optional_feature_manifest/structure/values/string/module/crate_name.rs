use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_module_crate_name_field_calls_raw_parser() {
    assert!(
        STRING_MODULE_CRATE_NAME.contains("super::super::raw::string_from_plugin_toml(value)"),
        "module crate-name string field projection should call the raw parser from its child owner"
    );
}
