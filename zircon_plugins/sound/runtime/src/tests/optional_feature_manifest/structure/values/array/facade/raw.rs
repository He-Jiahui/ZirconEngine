use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_facade_does_not_reexport_raw_parser() {
    assert!(
        !ARRAY_ROOT.contains("use raw::string_array_values"),
        "array parent must not re-export the raw TOML parser"
    );
}
