use super::super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_section_line_feature_leaf_parses_current_feature_rows() {
    assert!(
        PARSER_STATE_SECTION_LINE_FEATURE.contains(
            "parse_optional_feature_line(line, current::required_current_feature(state))",
        ),
        "parser state section-line feature leaf should own current-feature row parsing"
    );
}
