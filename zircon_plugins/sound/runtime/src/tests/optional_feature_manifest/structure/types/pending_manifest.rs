use super::super::sources::*;

#[test]
fn optional_feature_pending_manifest_declaration_stays_split_from_facade() {
    assert!(
        TYPES_PENDING_MANIFEST.contains("struct PendingOptionalFeatureManifest")
            && TYPES_PENDING_MANIFEST.contains("id: Option<String>")
            && TYPES_PENDING_MANIFEST.contains("dependencies:")
            && TYPES_PENDING_MANIFEST.contains("modules:"),
        "pending manifest child should own scanner DTO declaration"
    );
}
