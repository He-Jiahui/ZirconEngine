use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_identity_crate_name_owner_composes_children() {
    assert!(
        PARSER_LINE_MODULE_IDENTITY_CRATE_NAME.contains("mod field;")
            && PARSER_LINE_MODULE_IDENTITY_CRATE_NAME.contains("mod state;")
            && PARSER_LINE_MODULE_IDENTITY_CRATE_NAME.contains("mod value;")
            && PARSER_LINE_MODULE_IDENTITY_CRATE_NAME
                .contains("let Some(value) = field::module_crate_name_value(line)")
            && PARSER_LINE_MODULE_IDENTITY_CRATE_NAME.contains("state::set_module_crate_name")
            && PARSER_LINE_MODULE_IDENTITY_CRATE_NAME
                .contains("value::module_crate_name_from_plugin_toml(value)"),
        "module identity crate-name owner should compose field, conversion, and state children"
    );
}
