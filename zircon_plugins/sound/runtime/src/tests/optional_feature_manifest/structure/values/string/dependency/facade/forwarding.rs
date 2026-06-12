use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_dependency_facade_does_not_own_forwarding_bodies() {
    assert!(
        !STRING_DEPENDENCY.contains("fn dependency_plugin_id_string_from_plugin_toml")
            && !STRING_DEPENDENCY.contains("fn dependency_capability_string_from_plugin_toml"),
        "dependency string domain must not own semantic field forwarding bodies"
    );
}
