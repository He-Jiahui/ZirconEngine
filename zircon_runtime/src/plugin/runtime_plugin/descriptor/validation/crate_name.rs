use super::super::super::package_validation::is_lowercase_runtime_plugin_token;

// 链接插件的 crate 标识须满足包模块命名约束，便于清单投影与后续构建定位同一实体。
pub(super) fn validate_runtime_plugin_descriptor_crate_name(
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if crate_name.is_empty()
        || has_outer_whitespace(crate_name)
        || !crate_name.starts_with("zircon_plugin_")
        || !is_lowercase_runtime_plugin_token(crate_name)
    {
        diagnostics.push(format!(
            "runtime plugin descriptor crate_name `{crate_name}` must use `zircon_plugin_` prefix and contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
    if crate_name.ends_with('_') || crate_name.contains("__") {
        diagnostics.push(format!(
            "runtime plugin descriptor crate_name `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

// ASCII 命名规则之外仍要拒绝两端的 Unicode 空白；这里只判定边界，完整字符集交由标识规则校验。
fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

#[cfg(test)]
#[path = "tests/crate_name_optimization_tests.rs"]
mod optimization_tests;
