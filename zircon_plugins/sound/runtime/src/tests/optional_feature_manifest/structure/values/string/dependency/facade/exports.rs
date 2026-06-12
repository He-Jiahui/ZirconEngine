use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_dependency_facade_reexports_field_helpers() {
    assert!(
        STRING_DEPENDENCY.contains("use capability::dependency_capability_string_from_plugin_toml")
            && STRING_DEPENDENCY
                .contains("use plugin_id::dependency_plugin_id_string_from_plugin_toml"),
        "dependency string domain should expose child-owned field helpers"
    );
}
