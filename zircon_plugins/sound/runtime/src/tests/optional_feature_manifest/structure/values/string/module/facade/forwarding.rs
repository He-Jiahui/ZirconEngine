use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_module_facade_does_not_own_forwarding_bodies() {
    assert!(
        !STRING_MODULE.contains("fn module_name_string_from_plugin_toml")
            && !STRING_MODULE.contains("fn module_crate_name_string_from_plugin_toml"),
        "module string domain must not own semantic field forwarding bodies"
    );
}
