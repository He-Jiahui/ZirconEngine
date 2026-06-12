use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_entry_facade_stays_structural() {
    assert!(
        PARSER_STATE_ENTRY.contains("mod finish;") && PARSER_STATE_ENTRY.contains("mod line;"),
        "parser state entry parent must remain a structural method-module owner"
    );
    assert!(
        !PARSER_STATE_ENTRY.contains("fn finish")
            && !PARSER_STATE_ENTRY.contains("fn parse_manifest_line")
            && !PARSER_STATE_ENTRY.contains("manifest.lines()"),
        "parser state entry parent must not own entry method bodies"
    );
}
