pub(super) fn validate_runtime_plugin_feature_token_underscore(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.ends_with('_') || value.contains("__") {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must not end with an underscore or contain repeated underscores"
        ));
    }
}
