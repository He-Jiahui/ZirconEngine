use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_feature_line_facade_stays_structural() {
    assert!(
        PARSER_LINE_FEATURE.contains("mod capabilities;")
            && PARSER_LINE_FEATURE.contains("mod defaults;")
            && PARSER_LINE_FEATURE.contains("mod dispatch;")
            && PARSER_LINE_FEATURE.contains("mod entry;")
            && PARSER_LINE_FEATURE.contains("mod identity;"),
        "parser feature-line parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_LINE_FEATURE.contains("fn parse_optional_feature_line")
            && !PARSER_LINE_FEATURE.contains("dispatch::parse_feature_line")
            && !PARSER_LINE_FEATURE.contains("PendingOptionalFeatureManifest"),
        "parser feature-line parent must not own the entry forwarding body or feature state signature"
    );
    assert!(
        PARSER_LINE_FEATURE.contains("use entry::parse_optional_feature_line"),
        "parser feature-line parent should expose the entry child through a re-export"
    );
}
