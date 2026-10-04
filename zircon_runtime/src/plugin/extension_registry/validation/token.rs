// 包级 token 用于 owner 和清单键，限制字符集以减少跨模块映射歧义。
pub(super) fn is_lowercase_plugin_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

// 点分包 ID 另限制首字符、空段和外侧空白；由组件与清单验证共用。
pub(super) fn is_lowercase_plugin_package_id(value: &str) -> bool {
    !value.is_empty()
        && !has_outer_whitespace(value)
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value.split('.').all(is_lowercase_plugin_package_segment)
}

fn has_outer_whitespace(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

fn is_lowercase_plugin_package_segment(value: &str) -> bool {
    is_lowercase_plugin_token(value) && !value.ends_with('_') && !value.contains("__")
}

#[cfg(test)]
#[path = "tests/token_optimization_tests.rs"]
mod optimization_tests;
