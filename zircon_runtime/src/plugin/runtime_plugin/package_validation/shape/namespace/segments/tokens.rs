use super::super::super::token::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_package_namespace_segment_tokens(
    field_name: &str,
    value: &str,
    segments: &[&str],
    diagnostics: &mut Vec<String>,
) {
    if segments
        .iter()
        .any(|segment| !is_lowercase_runtime_plugin_token(segment))
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
        ));
    }
}
