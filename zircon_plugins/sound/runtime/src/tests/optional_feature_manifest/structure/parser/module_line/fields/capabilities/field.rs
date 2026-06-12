use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_capabilities_field_child_extracts_bracketed_list() {
    assert!(
        PARSER_LINE_MODULE_CAPABILITIES_FIELD
            .contains("bracketed_value(line, \"capabilities = [\")"),
        "module capabilities field child should own bracketed capability-list extraction"
    );
}
