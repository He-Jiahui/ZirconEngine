use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_facade_reexports_semantic_list_helpers() {
    assert!(
        ARRAY_ROOT.contains("use feature::feature_capability_list_from_plugin_toml")
            && ARRAY_ROOT.contains("use list::string_list_from_plugin_toml")
            && ARRAY_ROOT.contains("use module::module_capability_list_from_plugin_toml"),
        "array parent should expose semantic list helpers through child re-exports"
    );
}
