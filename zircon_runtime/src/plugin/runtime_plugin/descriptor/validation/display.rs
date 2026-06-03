pub(super) fn validate_runtime_plugin_display_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.trim().is_empty() || value.trim() != value {
        diagnostics.push(format!(
            "runtime plugin descriptor {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}
