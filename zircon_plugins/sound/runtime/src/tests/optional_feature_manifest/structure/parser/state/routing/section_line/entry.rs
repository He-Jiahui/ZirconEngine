use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_section_line_entry_owner_routes_active_section() {
    assert!(
        PARSER_STATE_SECTION_LINE_ENTRY.contains("pub(in super::super) fn parse_section_line")
            && PARSER_STATE_SECTION_LINE_ENTRY.contains("match state.section")
            && PARSER_STATE_SECTION_LINE_ENTRY
                .contains("super::feature::parse_feature_section_line")
            && PARSER_STATE_SECTION_LINE_ENTRY
                .contains("super::dependency::parse_dependency_section_line")
            && PARSER_STATE_SECTION_LINE_ENTRY.contains("super::module::parse_module_section_line"),
        "parser state section-line entry child should own active-section routing"
    );
}
