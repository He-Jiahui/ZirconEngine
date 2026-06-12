use super::super::super::super::sources::*;

#[test]
fn optional_feature_string_facade_does_not_own_field_forwarding_bodies() {
    assert!(
        !STRING_ROOT.contains("fn dependency_plugin_id_string_from_plugin_toml")
            && !STRING_ROOT.contains("fn dependency_capability_string_from_plugin_toml")
            && !STRING_ROOT.contains("fn feature_id_string_from_plugin_toml")
            && !STRING_ROOT.contains("fn feature_display_name_string_from_plugin_toml")
            && !STRING_ROOT.contains("fn feature_owner_plugin_string_from_plugin_toml")
            && !STRING_ROOT.contains("fn module_name_string_from_plugin_toml")
            && !STRING_ROOT.contains("fn module_crate_name_string_from_plugin_toml"),
        "string parent must not own semantic field forwarding bodies"
    );
}
