pub(super) fn validate_runtime_plugin_feature_capability_presence(
    capabilities: &[String],
    diagnostics: &mut Vec<String>,
) {
    if capabilities.is_empty() {
        diagnostics.push(
            "runtime plugin feature manifest capabilities must declare at least one capability"
                .to_string(),
        );
    }
}
