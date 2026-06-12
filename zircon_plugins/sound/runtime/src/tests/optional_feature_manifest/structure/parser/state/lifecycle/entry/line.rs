use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_entry_line_child_owns_manifest_line_dispatch() {
    assert!(
        PARSER_STATE_ENTRY_LINE.contains("impl OptionalFeatureParserState")
            && PARSER_STATE_ENTRY_LINE.contains("fn parse_manifest_line")
            && PARSER_STATE_ENTRY_LINE.contains("OptionalFeatureSection::from_table_header")
            && PARSER_STATE_ENTRY_LINE.contains("transition::enter_section")
            && PARSER_STATE_ENTRY_LINE.contains("section_line::parse_section_line"),
        "parser state line child should own table-header dispatch and section-line fallback"
    );
}
