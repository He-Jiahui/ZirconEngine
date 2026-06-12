use super::super::super::sources::*;

#[test]
fn optional_feature_parser_line_field_raw_owner_strips_prefix() {
    assert!(
        PARSER_LINE_FIELD_RAW.contains("line.strip_prefix(prefix)"),
        "raw field child should own prefix stripping"
    );
}
