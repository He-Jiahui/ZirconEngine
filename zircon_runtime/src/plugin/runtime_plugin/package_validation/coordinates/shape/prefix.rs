use super::super::super::is_lowercase_runtime_plugin_token;

pub(in crate::plugin::runtime_plugin::package_validation::coordinates) fn validate_runtime_plugin_package_coordinate_prefix(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.trim().is_empty()
        || value.trim() != value
        || value
            .split('.')
            .any(|segment| !is_lowercase_runtime_plugin_token(segment))
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must contain only non-empty lowercase coordinate segments"
        ));
    }
}
