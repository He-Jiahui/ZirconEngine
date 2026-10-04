//! 在注册报告生成期间检查包贡献声明的重复身份与归属；本阶段不写入运行时扩展注册表。
mod groups;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_contributions(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    groups::validate_runtime_plugin_package_contribution_groups(
        package_manifest,
        projection,
        diagnostics,
    );
}
