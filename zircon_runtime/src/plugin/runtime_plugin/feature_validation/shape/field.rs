pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.trim().is_empty() || value.trim() != value {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}
