//! 完整包校验中的嵌入特性阶段：包类型限制、特性自身合法性、提供者重复和包级目标覆盖分别累计诊断。
//! 包类型出错后仍扫描两类列表，以便一次报告清单中的其他问题。
mod kind;
mod lists;
mod manifest;
mod row;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_embedded_features(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    kind::validate_runtime_plugin_package_feature_kind(package_manifest, diagnostics);
    lists::validate_runtime_plugin_package_feature_lists(package_manifest, projection, diagnostics);
}
