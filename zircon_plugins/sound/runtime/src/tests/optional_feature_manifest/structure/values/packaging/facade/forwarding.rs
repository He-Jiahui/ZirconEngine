use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_facade_does_not_own_default_forwarding_body() {
    assert!(
        !PACKAGING_ROOT.contains("fn default_packaging_strategy_list_from_plugin_toml"),
        "packaging parent must not own semantic default-packaging forwarding bodies"
    );
}
