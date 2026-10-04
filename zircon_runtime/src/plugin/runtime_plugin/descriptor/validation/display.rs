// 用户可见名称在形成注册报告前校验，避免空白或外侧空白进入目录显示与诊断。
pub(super) fn validate_runtime_plugin_display_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() != value.len() {
        diagnostics.push(format!(
            "runtime plugin descriptor {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}

#[cfg(test)]
#[path = "display/tests/single_trim_tests.rs"]
mod single_trim_tests;
