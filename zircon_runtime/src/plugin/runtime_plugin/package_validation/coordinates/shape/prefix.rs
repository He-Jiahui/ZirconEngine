use super::super::super::is_lowercase_runtime_plugin_token;

/// 前缀允许多个点分段，供包坐标组合使用；每段必须是与包标识一致的小写令牌。
pub(in crate::plugin::runtime_plugin::package_validation::coordinates) fn validate_runtime_plugin_package_coordinate_prefix(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if value.is_empty()
        || has_outer_whitespace(value)
        || value
            .split('.')
            .any(|segment| !is_lowercase_runtime_plugin_token(segment))
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` must contain only non-empty lowercase coordinate segments"
        ));
    }
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

#[cfg(test)]
#[path = "tests/prefix_optimization_tests.rs"]
mod optimization_tests;
