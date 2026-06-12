use super::super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_section_line_dependency_leaf_parses_dependency_rows() {
    assert!(
        PARSER_STATE_SECTION_LINE_DEPENDENCY.contains("parse_optional_feature_dependency_line"),
        "parser state section-line dependency leaf should own dependency row parsing"
    );
}
