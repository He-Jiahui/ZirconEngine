use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_facade_stays_structural() {
    assert!(
        PARSER_LINE_MODULE.contains("mod capabilities;")
            && PARSER_LINE_MODULE.contains("mod dispatch;")
            && PARSER_LINE_MODULE.contains("mod entry;")
            && PARSER_LINE_MODULE.contains("mod identity;")
            && PARSER_LINE_MODULE.contains("mod kind;")
            && PARSER_LINE_MODULE.contains("mod targets;"),
        "parser module-line parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_LINE_MODULE.contains("fn parse_optional_feature_module_line")
            && !PARSER_LINE_MODULE.contains("dispatch::parse_module_line")
            && !PARSER_LINE_MODULE.contains("Option<String>")
            && !PARSER_LINE_MODULE.contains("RuntimeTargetMode"),
        "parser module-line parent must not own the entry forwarding body or field state signature"
    );
    assert!(
        PARSER_LINE_MODULE.contains("use entry::parse_optional_feature_module_line"),
        "parser module-line parent should expose the entry child through a re-export"
    );
}
