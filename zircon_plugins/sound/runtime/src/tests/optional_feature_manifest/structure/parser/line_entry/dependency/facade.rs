use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_dependency_line_facade_stays_structural() {
    assert!(
        PARSER_LINE_DEPENDENCY.contains("mod dispatch;")
            && PARSER_LINE_DEPENDENCY.contains("mod entry;")
            && PARSER_LINE_DEPENDENCY.contains("mod identity;")
            && PARSER_LINE_DEPENDENCY.contains("mod primary;"),
        "parser dependency-line parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_LINE_DEPENDENCY.contains("fn parse_optional_feature_dependency_line")
            && !PARSER_LINE_DEPENDENCY.contains("dispatch::parse_dependency_line")
            && !PARSER_LINE_DEPENDENCY.contains("Option<String>")
            && !PARSER_LINE_DEPENDENCY.contains("Option<bool>"),
        "parser dependency-line parent must not own the entry forwarding body or dependency field state signature"
    );
    assert!(
        PARSER_LINE_DEPENDENCY.contains("use entry::parse_optional_feature_dependency_line"),
        "parser dependency-line parent should expose the entry child through a re-export"
    );
}
