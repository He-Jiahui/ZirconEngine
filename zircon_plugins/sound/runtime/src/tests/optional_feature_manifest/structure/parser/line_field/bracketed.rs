use super::super::super::sources::*;

#[test]
fn optional_feature_parser_line_field_bracketed_owner_strips_bracket_suffix() {
    assert!(
        PARSER_LINE_FIELD_BRACKETED.contains("super::raw::raw_value(line, prefix)")
            && PARSER_LINE_FIELD_BRACKETED.contains("strip_suffix(']')"),
        "bracketed field child should own bracket suffix stripping over the raw child"
    );
}
