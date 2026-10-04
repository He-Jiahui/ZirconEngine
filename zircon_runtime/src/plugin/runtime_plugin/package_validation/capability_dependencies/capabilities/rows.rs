use crate::plugin::PluginPackageManifest;

use super::super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_capability_row;

/// 保持原能力数组顺序，并复用包投影中的后续重复行标记，使诊断顺序不受集合遍历影响。
pub(super) fn validate_runtime_plugin_package_capability_rows(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, capability) in package_manifest.capabilities.iter().enumerate() {
        validate_runtime_plugin_package_capability_row(
            capability,
            projection.package_capability_is_duplicate(index),
            diagnostics,
        );
    }
}
