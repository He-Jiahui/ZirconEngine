use super::super::super::super::sources::*;

#[test]
fn optional_feature_boolean_facade_does_not_reexport_raw_parser() {
    assert!(
        !BOOLEAN_ROOT.contains("use raw::bool_from_plugin_toml"),
        "boolean parent must not re-export the raw TOML parser"
    );
}
