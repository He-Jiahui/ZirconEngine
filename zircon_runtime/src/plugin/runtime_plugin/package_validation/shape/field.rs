pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    if value.trim().is_empty() || value.trim() != value {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must be non-empty and trimmed"
        ));
        return false;
    }
    true
}
