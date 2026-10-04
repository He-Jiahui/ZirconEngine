mod segments;

/// 用于功能、能力和模块等点分标识符；分段允许数字或下划线开头，
/// 归属插件 ID 的首字母限制由独立的插件标记规则承担。
pub(in crate::plugin::runtime_plugin::feature_validation) fn validate_runtime_plugin_feature_namespace(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    segments::validate_runtime_plugin_feature_namespace_segments(field_name, value, diagnostics);
}
