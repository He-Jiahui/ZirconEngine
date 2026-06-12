use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_dependency_capability_field_calls_raw_parser() {
    assert!(
        STRING_DEPENDENCY_CAPABILITY.contains("super::super::raw::string_from_plugin_toml(value)"),
        "dependency capability string field projection should call the raw parser from its child owner"
    );
}
