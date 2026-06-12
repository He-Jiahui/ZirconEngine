use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_section_line_facade_stays_structural() {
    assert!(
        PARSER_STATE_SECTION_LINE.contains("mod dependency;")
            && PARSER_STATE_SECTION_LINE.contains("mod entry;")
            && PARSER_STATE_SECTION_LINE.contains("mod feature;")
            && PARSER_STATE_SECTION_LINE.contains("mod module;"),
        "parser state section-line parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_STATE_SECTION_LINE.contains("fn parse_section_line")
            && !PARSER_STATE_SECTION_LINE.contains("match state.section")
            && !PARSER_STATE_SECTION_LINE.contains("OptionalFeatureSection"),
        "parser state section-line parent must not own active-section routing"
    );
    assert!(
        PARSER_STATE_SECTION_LINE.contains("use entry::parse_section_line"),
        "parser state section-line parent should expose the entry child through a re-export"
    );
}
