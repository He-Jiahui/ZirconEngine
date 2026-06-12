use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_dispatch_owner_routes_fields() {
    assert!(
        PARSER_LINE_MODULE_DISPATCH.contains("identity::parse_module_identity_line")
            && PARSER_LINE_MODULE_DISPATCH.contains("kind::parse_module_kind_line")
            && PARSER_LINE_MODULE_DISPATCH.contains("targets::parse_module_target_modes_line")
            && PARSER_LINE_MODULE_DISPATCH.contains("capabilities::parse_module_capabilities_line"),
        "parser module-line dispatch child should own ordered module-field routing"
    );
}
