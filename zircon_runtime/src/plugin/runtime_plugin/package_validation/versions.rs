mod component;
mod field;
mod segments;

/// 包版本与 SDK API 版本都按严格的三段数字形式校验；此契约不接受带前后缀的完整 SemVer 表达式。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_semver(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    if !field::validate_runtime_plugin_package_semver_field(field_name, value, diagnostics) {
        return;
    }
    segments::validate_runtime_plugin_package_semver_segments(field_name, value, diagnostics);
}
