use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_defaults_facade_reexports_entry_helper() {
    assert!(
        PACKAGING_DEFAULTS.contains("use entry::default_packaging_strategy_list_from_plugin_toml"),
        "default packaging projection parent should expose the child-owned entry helper"
    );
}
