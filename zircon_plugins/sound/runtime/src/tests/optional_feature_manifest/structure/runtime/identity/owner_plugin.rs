use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_identity_owner_plugin_child_owns_projection() {
    assert!(
        RUNTIME_IDENTITY_OWNER_PLUGIN.contains("feature.owner_plugin_id.clone()"),
        "runtime identity owner-plugin child should own owner-plugin projection"
    );
}
