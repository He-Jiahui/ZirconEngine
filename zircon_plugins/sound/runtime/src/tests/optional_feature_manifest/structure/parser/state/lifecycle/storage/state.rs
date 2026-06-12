use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_storage_state_child_owns_scanner_fields() {
    assert!(
        PARSER_STATE_STORAGE.contains("struct OptionalFeatureParserState")
            && PARSER_STATE_STORAGE.contains("features: Vec<StaticOptionalFeatureManifest>")
            && PARSER_STATE_STORAGE
                .contains("current_feature: Option<PendingOptionalFeatureManifest>")
            && PARSER_STATE_STORAGE.contains("section: OptionalFeatureSection"),
        "parser state storage child should own scanner state fields"
    );
}
