use crate::plugin::PluginPackageManifest;

use super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_capability_status_row;

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
