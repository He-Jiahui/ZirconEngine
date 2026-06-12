use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_kind_value_child_converts_typed_kind() {
    assert!(
        PARSER_LINE_MODULE_KIND_VALUE.contains("module_kind_value_from_plugin_toml(value)"),
        "module kind value child should own typed module-kind conversion"
    );
}
