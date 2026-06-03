use super::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_package_token_charset(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.trim().is_empty() || value.trim() != value || !is_lowercase_runtime_plugin_token(value)
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
}
