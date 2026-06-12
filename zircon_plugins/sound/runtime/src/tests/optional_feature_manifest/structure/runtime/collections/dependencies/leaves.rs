use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_dependency_collection_leaves_own_projection_signature_and_ordering() {
    assert!(
        RUNTIME_DEPENDENCIES_PROJECTION.contains(".map(signature::dependency_signature)")
            && RUNTIME_DEPENDENCIES_SIGNATURE.contains("dependency.plugin_id.clone()")
            && RUNTIME_DEPENDENCIES_SIGNATURE.contains("dependency.capability.clone()")
            && RUNTIME_DEPENDENCIES_ORDERING.contains("dependencies.sort_unstable()"),
        "runtime dependencies projection, signature, and ordering children should own leaf behavior"
    );
}
