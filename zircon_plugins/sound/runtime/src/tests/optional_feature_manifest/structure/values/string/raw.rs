use super::super::super::sources::*;

#[test]
fn optional_feature_string_raw_parser_owner_stays_isolated() {
    assert!(
        STRING_RAW.contains("value.to_string()"),
        "string raw TOML parser must remain isolated in raw.rs"
    );
}
