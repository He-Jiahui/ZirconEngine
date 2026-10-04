mod presence;
mod row;
mod rows;
mod uniqueness;

use super::projection::RuntimePluginFeatureValidationProjection;

/// 校验功能对外承诺的能力集合；注册报告和包内嵌功能共用此入口。
/// 诊断累积到调用方的报告，能力重复信息必须来自同一清单顺序的投影。
pub(super) fn validate_runtime_plugin_feature_capabilities(
    capabilities: &[String],
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    presence::validate_runtime_plugin_feature_capability_presence(capabilities, diagnostics);
    rows::validate_runtime_plugin_feature_capability_rows(capabilities, projection, diagnostics);
}
