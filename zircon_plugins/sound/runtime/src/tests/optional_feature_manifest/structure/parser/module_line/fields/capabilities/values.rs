use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_capabilities_values_child_converts_string_list() {
    assert!(
        PARSER_LINE_MODULE_CAPABILITIES_VALUES
            .contains("module_capability_list_from_plugin_toml(value)"),
        "module capabilities values child should own string-list conversion"
    );
}
