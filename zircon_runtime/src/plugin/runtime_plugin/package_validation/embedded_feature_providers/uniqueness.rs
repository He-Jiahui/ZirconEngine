pub(super) fn validate_runtime_plugin_package_feature_provider_uniqueness(
    field_name: &str,
    feature_id: &str,
    provider_package_id: &str,
    seen_feature_providers: &mut Vec<(String, String)>,
    diagnostics: &mut Vec<String>,
) {
    let key = (feature_id.to_string(), provider_package_id.to_string());
    if seen_feature_providers.contains(&key) {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{feature_id}` provider `{provider_package_id}` must be unique",
        ));
        return;
    }
    seen_feature_providers.push(key);
}
