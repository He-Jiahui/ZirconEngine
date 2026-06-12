use super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_facade_reexports_semantic_helper() {
    assert!(
        TARGET_MODE_ROOT.contains("use module::module_target_mode_list_from_plugin_toml"),
        "target_mode parent should expose semantic helpers through child re-exports"
    );
}
