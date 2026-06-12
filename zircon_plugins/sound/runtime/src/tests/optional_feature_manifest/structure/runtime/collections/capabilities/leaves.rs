use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_capability_collection_leaves_own_projection_and_ordering() {
    assert!(
        RUNTIME_CAPABILITIES_PROJECTION.contains("feature.capabilities.clone()")
            && RUNTIME_CAPABILITIES_ORDERING.contains("capabilities.sort_unstable()"),
        "runtime capabilities projection and ordering children should own their leaf behavior"
    );
}
