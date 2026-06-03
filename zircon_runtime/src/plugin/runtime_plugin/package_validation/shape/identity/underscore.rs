pub(super) fn validate_runtime_plugin_package_id_underscore(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.ends_with('_') || value.contains("__") {
        diagnostics.push(format!(
            "{context} {field_name} `{value}` must not end with an underscore or contain repeated underscores"
        ));
    }
}
