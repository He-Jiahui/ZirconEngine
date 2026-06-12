use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_feature_line_dispatch_owner_routes_fields() {
    assert!(
        PARSER_LINE_FEATURE_DISPATCH.contains("identity::parse_feature_identity_line")
            && PARSER_LINE_FEATURE_DISPATCH.contains("capabilities::parse_feature_capability_line")
            && PARSER_LINE_FEATURE_DISPATCH.contains("defaults::parse_feature_defaults_line"),
        "parser feature-line dispatch child should own ordered feature-field routing"
    );
}
