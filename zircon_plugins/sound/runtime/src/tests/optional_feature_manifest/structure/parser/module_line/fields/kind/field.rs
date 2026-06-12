use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_kind_field_child_extracts_quoted_kind() {
    assert!(
        PARSER_LINE_MODULE_KIND_FIELD.contains("quoted_value(line, \"kind = \\\"\")"),
        "module kind field child should own quoted field extraction"
    );
}
