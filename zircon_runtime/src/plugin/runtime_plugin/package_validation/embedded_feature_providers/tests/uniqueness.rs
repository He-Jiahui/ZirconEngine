#[test]
fn feature_provider_uniqueness_borrows_identity_parts() {
    let source = include_str!("../uniqueness.rs");
    let owned_feature = ["feature_id", ".to_string()"].concat();
    let owned_provider = ["provider_package_id", ".to_string()"].concat();
    assert!(!source.contains(&owned_feature));
    assert!(!source.contains(&owned_provider));
}

#[test]
fn feature_provider_uniqueness_preserves_duplicate_diagnostics() {
    let mut diagnostics = Vec::new();
    super::validate_runtime_plugin_package_feature_provider_uniqueness(
        "optional feature",
        "rendering.deferred",
        "rendering",
        false,
        &mut diagnostics,
    );
    super::validate_runtime_plugin_package_feature_provider_uniqueness(
        "optional feature",
        "rendering.deferred",
        "rendering",
        true,
        &mut diagnostics,
    );
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].contains("must be unique"));
}
