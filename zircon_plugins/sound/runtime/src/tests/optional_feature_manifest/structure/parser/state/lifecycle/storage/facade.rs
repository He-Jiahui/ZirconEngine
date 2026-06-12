use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_storage_facade_stays_structural() {
    assert!(
        PARSER_STATE_ROOT.contains("mod entry;")
            && PARSER_STATE_ROOT.contains("mod flush;")
            && PARSER_STATE_ROOT.contains("mod section_line;")
            && PARSER_STATE_ROOT.contains("mod storage;")
            && PARSER_STATE_ROOT.contains("mod transition;"),
        "parser state parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_STATE_ROOT.contains("struct OptionalFeatureParserState")
            && !PARSER_STATE_ROOT.contains("fn parse_manifest_line")
            && !PARSER_STATE_ROOT.contains("fn finish")
            && !PARSER_STATE_ROOT.contains("fn flush_pending"),
        "parser state parent must not own storage, entry methods, or flush behavior"
    );
    assert!(
        PARSER_STATE_ROOT.contains("use self::storage::OptionalFeatureParserState"),
        "parser state parent should expose the storage-owned parser state type"
    );
}
