use super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_raw_parser_owner_stays_isolated() {
    assert!(
        TARGET_MODE_RAW.contains("match value.as_str()"),
        "target-mode raw TOML parser must remain isolated in raw.rs"
    );
}
