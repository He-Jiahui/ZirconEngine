use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_facade_does_not_expose_raw_parser() {
    assert!(
        !PACKAGING_ROOT.contains("use raw::packaging_strategy_from_plugin_toml")
            && !PACKAGING_ROOT.contains("fn packaging_strategy_from_plugin_toml"),
        "packaging parent must not expose the raw TOML parser"
    );
}
