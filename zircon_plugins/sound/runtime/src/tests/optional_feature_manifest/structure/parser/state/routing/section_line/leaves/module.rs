use super::super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_section_line_module_leaf_parses_module_rows() {
    assert!(
        PARSER_STATE_SECTION_LINE_MODULE.contains("parse_optional_feature_module_line"),
        "parser state section-line module leaf should own module row parsing"
    );
}
