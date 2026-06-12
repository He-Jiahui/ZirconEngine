use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_feature_line_entry_owner_forwards_to_dispatch() {
    assert!(
        PARSER_LINE_FEATURE_ENTRY
            .contains("pub(in super::super::super) fn parse_optional_feature_line")
            && PARSER_LINE_FEATURE_ENTRY.contains("super::dispatch::parse_feature_line"),
        "parser feature-line entry child should own the facade-visible dispatch handoff"
    );
}
