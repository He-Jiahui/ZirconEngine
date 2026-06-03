mod charset;
mod predicate;

pub(in crate::plugin::runtime_plugin) use predicate::is_lowercase_runtime_plugin_token;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_token(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    charset::validate_runtime_plugin_package_token_charset(field_name, value, diagnostics);
}
