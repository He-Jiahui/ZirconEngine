use super::super::super::sources::*;

#[test]
fn optional_feature_packaging_raw_parser_owner_stays_isolated() {
    assert!(
        PACKAGING_RAW.contains("match value.as_str()"),
        "packaging raw TOML parser must remain isolated in raw.rs"
    );
}
