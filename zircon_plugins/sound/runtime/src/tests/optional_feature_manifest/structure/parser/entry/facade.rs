use super::super::super::sources::*;

#[test]
fn optional_feature_parser_entry_facade_stays_split_from_manifest_scan() {
    assert!(
        PARSER_ROOT.contains("mod entry;")
            && PARSER_ROOT.contains("mod line;")
            && PARSER_ROOT.contains("mod pending;")
            && PARSER_ROOT.contains("mod section;")
            && PARSER_ROOT.contains("mod state;"),
        "parser parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_ROOT.contains("OptionalFeatureParserState")
            && !PARSER_ROOT.contains("manifest.lines()")
            && !PARSER_ROOT.contains("fn optional_features_from_plugin_toml"),
        "parser parent must not own scanner state lifecycle or manifest line iteration"
    );
    assert!(
        PARSER_ROOT.contains("use entry::optional_features_from_plugin_toml"),
        "parser parent should expose manifest parsing through the entry child re-export"
    );
}
