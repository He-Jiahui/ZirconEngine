use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_entry_owner_forwards_to_dispatch() {
    assert!(
        PARSER_LINE_MODULE_ENTRY
            .contains("pub(in super::super::super) fn parse_optional_feature_module_line")
            && PARSER_LINE_MODULE_ENTRY.contains("super::dispatch::parse_module_line"),
        "parser module-line entry child should own the facade-visible dispatch handoff"
    );
}
