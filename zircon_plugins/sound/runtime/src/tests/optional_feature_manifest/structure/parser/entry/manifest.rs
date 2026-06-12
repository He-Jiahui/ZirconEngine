use super::super::super::sources::*;

#[test]
fn optional_feature_parser_entry_manifest_owner_scans_lines() {
    assert!(
        PARSER_ENTRY.contains("OptionalFeatureParserState::default()")
            && PARSER_ENTRY.contains("manifest.lines().map(str::trim)")
            && PARSER_ENTRY.contains("parser.parse_manifest_line(line)")
            && PARSER_ENTRY.contains("parser.finish()"),
        "parser entry child should own scanner state lifecycle and manifest line iteration"
    );
}
