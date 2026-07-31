mod identity;
mod note;
mod row;
mod rows;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capability_statuses(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_capability_status_rows(
        package_manifest,
        projection,
        diagnostics,
    );
}
