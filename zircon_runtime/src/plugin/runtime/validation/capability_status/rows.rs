use crate::plugin::PluginPackageManifest;

use super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_capability_status_row;

/// 行号保持原清单顺序，既定位状态重复，也限定其参考路径重复检查的作用域。
pub(super) fn validate_runtime_plugin_package_capability_status_rows(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (status_index, status) in package_manifest.capability_statuses.iter().enumerate() {
        validate_runtime_plugin_package_capability_status_row(
            package_manifest,
            status,
            status_index,
            projection,
            diagnostics,
        );
    }
}
