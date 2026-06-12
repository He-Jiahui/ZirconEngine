use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_identity_facade_stays_structural() {
    assert!(
        PARSER_LINE_MODULE_IDENTITY.contains("mod crate_name;")
            && PARSER_LINE_MODULE_IDENTITY.contains("mod name;"),
        "module identity parent must remain a structural field-family owner"
    );
    assert!(
        !PARSER_LINE_MODULE_IDENTITY.contains("module_name_value")
            && !PARSER_LINE_MODULE_IDENTITY.contains("module_crate_name_value")
            && !PARSER_LINE_MODULE_IDENTITY.contains("set_module_name")
            && !PARSER_LINE_MODULE_IDENTITY.contains("set_module_crate_name"),
        "module identity parent must not own field extraction or state mutation"
    );
    assert!(
        PARSER_LINE_MODULE_IDENTITY.contains("name::parse_module_name_line")
            && PARSER_LINE_MODULE_IDENTITY.contains("crate_name::parse_module_crate_name_line"),
        "module identity parent should only route name and crate-name field owners"
    );
}
