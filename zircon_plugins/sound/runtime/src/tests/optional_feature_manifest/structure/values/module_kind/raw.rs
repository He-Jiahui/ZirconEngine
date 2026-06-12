use super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_raw_parser_owner_stays_isolated() {
    assert!(
        MODULE_KIND_RAW.contains("match value"),
        "module-kind raw TOML parser must remain isolated in raw.rs"
    );
}
