use super::super::super::super::package_validation::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_feature_token_charset(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !is_lowercase_runtime_plugin_token(value) {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
}
