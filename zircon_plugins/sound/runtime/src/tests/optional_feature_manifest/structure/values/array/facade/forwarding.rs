use super::super::super::super::sources::*;

#[test]
fn optional_feature_array_facade_does_not_own_capability_forwarding_bodies() {
    assert!(
        !ARRAY_ROOT.contains("fn feature_capability_list_from_plugin_toml")
            && !ARRAY_ROOT.contains("fn module_capability_list_from_plugin_toml"),
        "array parent must not own semantic capability-list forwarding bodies"
    );
}
