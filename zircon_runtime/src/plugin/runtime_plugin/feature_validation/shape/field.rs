/// 对显示名和机器字段共同要求非空且没有首尾空白；允许字段内部的展示空白。
/// 原值保持不变，调用方继续完成相应标识符规则并汇总所有诊断。
pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_field(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() != value.len() {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must be non-empty and trimmed"
        ));
    }
}

#[cfg(test)]
#[path = "field/tests/single_trim_tests.rs"]
mod single_trim_tests;
