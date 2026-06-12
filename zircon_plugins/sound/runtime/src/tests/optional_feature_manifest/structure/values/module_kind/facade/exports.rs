use super::super::super::super::sources::*;

#[test]
fn optional_feature_module_kind_facade_reexports_semantic_helper() {
    assert!(
        MODULE_KIND_ROOT.contains("use module::module_kind_value_from_plugin_toml"),
        "module_kind parent should expose semantic helpers through child re-exports"
    );
}
