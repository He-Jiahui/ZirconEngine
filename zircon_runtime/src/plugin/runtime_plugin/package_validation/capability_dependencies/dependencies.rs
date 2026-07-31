mod capability;
mod pairs;
mod row;
mod rows;

use super::super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_dependency_rows(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_dependency_rows(
        package_manifest,
        projection,
        diagnostics,
    );
}
