use crate::plugin::PluginPackageManifest;

use super::{presence, shape};

// 缺一项的坐标先报告完整性，再继续检查已有字段的形状，让注册报告一次列出可修复的问题。
pub(super) fn validate_runtime_plugin_package_coordinate_fields(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if !presence::validate_runtime_plugin_package_coordinate_presence(package_manifest, diagnostics)
    {
        return;
    }

    shape::validate_runtime_plugin_package_coordinate_prefix(
        "package_prefix",
        &package_manifest.package_prefix,
        diagnostics,
    );
    shape::validate_runtime_plugin_package_coordinate_segment(
        "package_company",
        &package_manifest.package_company,
        diagnostics,
    );
    shape::validate_runtime_plugin_package_coordinate_segment(
        "package_name",
        &package_manifest.package_name,
        diagnostics,
    );
}
