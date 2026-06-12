use super::super::super::sources::*;

#[test]
fn optional_feature_parser_section_facade_stays_split_from_routing() {
    assert!(
        PARSER_SECTION.contains("mod kind;") && PARSER_SECTION.contains("mod table_header;"),
        "parser section parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_SECTION.contains("enum OptionalFeatureSection")
            && !PARSER_SECTION.contains("fn from_table_header")
            && !PARSER_SECTION.contains("match line"),
        "parser section parent must not own section declaration or table-header classification"
    );
    assert!(
        PARSER_SECTION.contains("pub(super) use self::kind::OptionalFeatureSection"),
        "parser section parent should expose the section enum through the kind child re-export"
    );
}
