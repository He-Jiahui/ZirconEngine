use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_signature_facade_stays_structural() {
    assert!(
        RUNTIME_SIGNATURE.contains("mod entry;"),
        "runtime signature parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_SIGNATURE.contains("StaticOptionalFeatureManifest")
            && !RUNTIME_SIGNATURE.contains("id: super::"),
        "runtime signature parent must not own full optional-feature signature assembly"
    );
    assert!(
        RUNTIME_SIGNATURE.contains("use entry::optional_feature_signature"),
        "runtime signature parent should expose the child-owned entry point"
    );
}
