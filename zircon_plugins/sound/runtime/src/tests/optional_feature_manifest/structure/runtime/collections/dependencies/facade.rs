use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_dependency_collection_facade_stays_structural() {
    assert!(
        RUNTIME_DEPENDENCIES_ROOT.contains("mod entry;")
            && RUNTIME_DEPENDENCIES_ROOT.contains("mod ordering;")
            && RUNTIME_DEPENDENCIES_ROOT.contains("mod projection;")
            && RUNTIME_DEPENDENCIES_ROOT.contains("mod signature;"),
        "runtime dependencies parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_DEPENDENCIES_ROOT.contains("fn dependency_signatures"),
        "runtime dependencies parent must not own projection and ordering composition"
    );
    assert!(
        RUNTIME_DEPENDENCIES_ROOT.contains("use entry::dependency_signatures"),
        "runtime dependencies parent should expose the child-owned entry point"
    );
}
