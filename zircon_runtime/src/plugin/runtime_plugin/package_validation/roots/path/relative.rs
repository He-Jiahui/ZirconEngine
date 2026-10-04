// BUG: [CR-PLUGIN-VALIDATION-0360] Windows 盘符根如 C:/assets 与 C:assets 会通过相对路径校验；证据：包根的字段、分隔符和路径段检查均不识别盘符前缀。
pub(super) fn validate_runtime_plugin_package_root_relative(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if root
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'/' | b'\\'))
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be relative"
        ));
    }
}

#[cfg(test)]
#[path = "tests/relative.rs"]
mod tests;
