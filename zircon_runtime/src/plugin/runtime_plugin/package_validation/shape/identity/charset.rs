use super::super::token::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_package_id_charset(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.trim().is_empty()
        || value.trim() != value
        || !is_lowercase_runtime_plugin_package_id(value)
    {
        diagnostics.push(format!(
            "{context} {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments"
        ));
    }
}

fn is_lowercase_runtime_plugin_package_id(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_lowercase_runtime_plugin_token)
}
