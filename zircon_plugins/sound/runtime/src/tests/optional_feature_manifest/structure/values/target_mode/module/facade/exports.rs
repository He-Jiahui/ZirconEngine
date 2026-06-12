use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_module_facade_reexports_entry_helper() {
    assert!(
        TARGET_MODE_MODULE.contains("use entry::module_target_mode_list_from_plugin_toml"),
        "module target-mode projection parent should expose the child-owned entry helper"
    );
}
