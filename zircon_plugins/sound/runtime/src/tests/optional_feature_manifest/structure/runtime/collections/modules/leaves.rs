use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_module_collection_leaves_own_projection_signature_and_ordering() {
    assert!(
        RUNTIME_MODULES_PROJECTION.contains(".map(signature::module_signature)")
            && RUNTIME_MODULES_SIGNATURE.contains("module.name.clone()")
            && RUNTIME_MODULES_SIGNATURE.contains("module.capabilities.clone()")
            && RUNTIME_MODULES_ORDERING.contains("modules.sort_unstable_by_key"),
        "runtime modules projection, signature, and ordering children should own leaf behavior"
    );
}
