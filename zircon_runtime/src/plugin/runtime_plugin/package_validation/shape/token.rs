mod charset;
mod predicate;

pub(in crate::plugin::runtime_plugin) use predicate::is_lowercase_runtime_plugin_token;

/// 令牌规则用于接口方法、参数等单段名称；它不承担整个包 ID 的点分结构约束。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_token(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    charset::validate_runtime_plugin_package_token_charset(field_name, value, diagnostics);
}
