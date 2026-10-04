mod row;
mod rows;

use super::projection::RuntimePluginFeatureValidationProjection;
use crate::plugin::PluginFeatureBundleManifest;

/// 允许没有模块的纯能力功能；有模块时统一审查模块身份、能力与目标模式。
/// 能否在某个产品目标启用，仍需目录将运行时模块与项目选择一起解析。
pub(super) fn validate_runtime_plugin_feature_modules(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_feature_module_rows(feature, projection, diagnostics);
}
