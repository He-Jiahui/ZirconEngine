use super::super::sources::*;

#[test]
fn optional_feature_dependency_signature_declaration_stays_split_from_facade() {
    assert!(
        TYPES_DEPENDENCY_SIGNATURE.contains("type OptionalFeatureDependencySignature")
            && TYPES_DEPENDENCY_SIGNATURE.contains("(String, String, bool)"),
        "dependency signature child should own the dependency tuple declaration"
    );
}
