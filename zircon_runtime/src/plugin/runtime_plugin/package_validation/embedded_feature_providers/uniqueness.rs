pub(super) fn validate_runtime_plugin_package_feature_provider_uniqueness(
    field_name: &str,
    feature_id: &str,
    provider_package_id: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{feature_id}` provider `{provider_package_id}` must be unique",
        ));
    }
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
}
