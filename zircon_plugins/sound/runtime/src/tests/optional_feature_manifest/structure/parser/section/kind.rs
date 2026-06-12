use super::super::super::sources::*;

#[test]
fn optional_feature_parser_section_kind_owner_declares_scanner_state() {
    assert!(
        PARSER_SECTION_KIND.contains("enum OptionalFeatureSection")
            && PARSER_SECTION_KIND.contains("None")
            && PARSER_SECTION_KIND.contains("Feature")
            && PARSER_SECTION_KIND.contains("Dependency")
            && PARSER_SECTION_KIND.contains("Module")
            && PARSER_SECTION_KIND.contains("impl Default for OptionalFeatureSection"),
        "parser section kind child should own section declaration and default scanner state"
    );
}
