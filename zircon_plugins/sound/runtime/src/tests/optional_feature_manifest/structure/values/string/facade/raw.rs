use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_facade_does_not_reexport_raw_parser() {
    assert!(
        !STRING_ROOT.contains("use raw::string_from_plugin_toml"),
        "string parent must not re-export the raw TOML parser"
    );
}
