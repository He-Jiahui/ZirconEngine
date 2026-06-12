use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_facade_does_not_expose_raw_parser() {
    assert!(
        !TARGET_MODE_ROOT.contains("use raw::runtime_target_mode_from_plugin_toml")
            && !TARGET_MODE_ROOT.contains("fn runtime_target_mode_from_plugin_toml"),
        "target_mode parent must not expose the raw TOML parser"
    );
}
