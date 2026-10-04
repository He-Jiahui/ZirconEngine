use super::super::token::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_package_id_charset(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.is_empty()
        || has_outer_whitespace(value)
        || !is_lowercase_runtime_plugin_package_id(value)
    {
        diagnostics.push(format!(
            "{context} {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments"
        ));
    }
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

fn is_lowercase_runtime_plugin_package_id(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_lowercase_runtime_plugin_token)
}

#[cfg(test)]
#[path = "tests/charset_optimization_tests.rs"]
mod optimization_tests;
