use crate::plugin::PluginPackageManifest;

use super::super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_capability_row;

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
