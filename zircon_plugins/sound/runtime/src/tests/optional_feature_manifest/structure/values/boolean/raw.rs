use super::super::super::sources::*;

#[test]
fn optional_feature_boolean_raw_parser_owner_stays_isolated() {
    assert!(
        BOOLEAN_RAW.contains("match value"),
        "boolean raw TOML parser must remain isolated in raw.rs"
    );
}
