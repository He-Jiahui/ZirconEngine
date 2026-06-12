use super::super::super::sources::*;

#[test]
fn optional_feature_array_raw_parser_owner_stays_isolated() {
    assert!(
        ARRAY_RAW.contains(".split(',')"),
        "array raw TOML parser must remain isolated in raw.rs"
    );
}
