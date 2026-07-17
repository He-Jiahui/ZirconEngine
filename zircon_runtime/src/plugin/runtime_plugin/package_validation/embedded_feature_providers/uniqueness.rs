pub(super) fn validate_runtime_plugin_package_feature_provider_uniqueness<'a>(
    field_name: &str,
    feature_id: &'a str,
    provider_package_id: &'a str,
    seen_feature_providers: &mut Vec<(&'a str, &'a str)>,
    diagnostics: &mut Vec<String>,
) {
    let key = (feature_id, provider_package_id);
    if seen_feature_providers.contains(&key) {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{feature_id}` provider `{provider_package_id}` must be unique",
        ));
        return;
    }
    seen_feature_providers.push(key);
}

#[cfg(test)]
mod tests {
    #[test]
    fn feature_provider_uniqueness_borrows_identity_parts() {
        let source = include_str!("uniqueness.rs");
        let owned_feature = ["feature_id", ".to_string()"].concat();
        let owned_provider = ["provider_package_id", ".to_string()"].concat();
        assert!(!source.contains(&owned_feature));
        assert!(!source.contains(&owned_provider));
    }

    #[test]
    fn feature_provider_uniqueness_preserves_duplicate_diagnostics() {
        let mut seen = Vec::new();
        let mut diagnostics = Vec::new();
        super::validate_runtime_plugin_package_feature_provider_uniqueness(
            "optional feature",
            "rendering.deferred",
            "rendering",
            &mut seen,
            &mut diagnostics,
        );
        super::validate_runtime_plugin_package_feature_provider_uniqueness(
            "optional feature",
            "rendering.deferred",
            "rendering",
            &mut seen,
            &mut diagnostics,
        );
        assert_eq!(seen, vec![("rendering.deferred", "rendering")]);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("must be unique"));
    }
}
