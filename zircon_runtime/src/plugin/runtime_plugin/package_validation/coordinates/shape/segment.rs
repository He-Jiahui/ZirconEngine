use super::super::super::is_lowercase_runtime_plugin_token;

pub(in crate::plugin::runtime_plugin::package_validation::coordinates) fn validate_runtime_plugin_package_coordinate_segment(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.trim().is_empty() || value.trim() != value || !is_lowercase_runtime_plugin_token(value)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must be a non-empty lowercase coordinate segment"
        ));
    }
}
