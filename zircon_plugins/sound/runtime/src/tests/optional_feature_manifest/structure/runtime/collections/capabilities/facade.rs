use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_capability_collection_facade_stays_structural() {
    assert!(
        RUNTIME_CAPABILITIES_ROOT.contains("mod entry;")
            && RUNTIME_CAPABILITIES_ROOT.contains("mod ordering;")
            && RUNTIME_CAPABILITIES_ROOT.contains("mod projection;"),
        "runtime capabilities parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_CAPABILITIES_ROOT.contains("fn capability_signatures"),
        "runtime capabilities parent must not own projection and ordering composition"
    );
    assert!(
        RUNTIME_CAPABILITIES_ROOT.contains("use entry::capability_signatures"),
        "runtime capabilities parent should expose the child-owned entry point"
    );
}
