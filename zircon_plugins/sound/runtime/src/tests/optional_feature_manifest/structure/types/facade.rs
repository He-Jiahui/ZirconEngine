use super::super::sources::*;

#[test]
fn optional_feature_type_facade_stays_split_from_declarations() {
    assert!(
        TYPES_ROOT.contains("mod dependency_signature;")
            && TYPES_ROOT.contains("mod module_signature;")
            && TYPES_ROOT.contains("mod pending_manifest;")
            && TYPES_ROOT.contains("mod static_manifest;"),
        "optional-feature type parent must remain a structural child-module owner"
    );
    assert!(
        !TYPES_ROOT.contains("type OptionalFeatureDependencySignature")
            && !TYPES_ROOT.contains("type OptionalFeatureModuleSignature")
            && !TYPES_ROOT.contains("struct PendingOptionalFeatureManifest")
            && !TYPES_ROOT.contains("struct StaticOptionalFeatureManifest"),
        "optional-feature type parent must not own tuple or DTO declarations"
    );
    assert!(
        TYPES_ROOT.contains("use self::dependency_signature::OptionalFeatureDependencySignature")
            && TYPES_ROOT.contains("use self::module_signature::OptionalFeatureModuleSignature")
            && TYPES_ROOT.contains("use self::pending_manifest::PendingOptionalFeatureManifest")
            && TYPES_ROOT.contains("use self::static_manifest::StaticOptionalFeatureManifest"),
        "optional-feature type parent should expose child-owned declarations through re-exports"
    );
}
