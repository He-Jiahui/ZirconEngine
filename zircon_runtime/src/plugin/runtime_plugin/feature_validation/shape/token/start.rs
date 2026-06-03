pub(super) fn validate_runtime_plugin_feature_token_start(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase())
    {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must start with a lowercase ASCII letter"
        ));
    }
}
