use super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_facade_reexports_semantic_helper() {
    assert!(
        PACKAGING_ROOT.contains("use defaults::default_packaging_strategy_list_from_plugin_toml"),
        "packaging parent should expose semantic helpers through child re-exports"
    );
}
