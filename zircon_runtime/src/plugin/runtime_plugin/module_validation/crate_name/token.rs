use super::super::super::package_validation::is_lowercase_runtime_plugin_token;

/// 固定插件前缀与 ASCII 字符集是 crate 标识契约；下划线边界由同级规则补足。
/// 不能把本规则无诊断当作完整有效性结论，例如只有固定前缀仍会被后续规则拒绝。
pub(super) fn validate_runtime_plugin_module_crate_name_token(
    manifest_label: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    const PREFIX: &[u8] = b"zircon_plugin_";
    let bytes = crate_name.as_bytes();
    let valid = bytes.len() >= PREFIX.len()
        && bytes.starts_with(PREFIX)
        && bytes[PREFIX.len()..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_');

    if !valid {
        diagnostics.push(format!(
            "{manifest_label} module crate_name `{crate_name}` must use `zircon_plugin_` prefix and contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
}

#[cfg(test)]
#[path = "tests/token.rs"]
mod tests;
