pub(super) fn validate_runtime_plugin_package_id_underscore(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value
        .split('.')
        .any(|segment| segment.ends_with('_') || segment.contains("__"))
    {
        diagnostics.push(format!(
            "{context} {field_name} `{value}` segments must not end with an underscore or contain repeated underscores"
        ));
    }
}
