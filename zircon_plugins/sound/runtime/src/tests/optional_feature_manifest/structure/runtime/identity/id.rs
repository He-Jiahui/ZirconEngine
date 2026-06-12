use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_identity_id_child_owns_projection() {
    assert!(
        RUNTIME_IDENTITY_ID.contains("feature.id.clone()"),
        "runtime identity id child should own id projection"
    );
}
