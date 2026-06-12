use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_defaults_facade_does_not_own_forwarding_body() {
    assert!(
        !PACKAGING_DEFAULTS.contains("fn default_packaging_strategy_list_from_plugin_toml")
            && !PACKAGING_DEFAULTS
                .contains("super::list::packaging_strategy_list_from_plugin_toml(value)"),
        "default packaging projection parent must not own semantic forwarding bodies"
    );
}
