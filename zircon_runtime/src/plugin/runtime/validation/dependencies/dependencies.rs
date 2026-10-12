#[path = "capability.rs"]
mod capability;
#[path = "dependencies/pairs.rs"]
mod pairs;
#[path = "dependencies/row.rs"]
mod row;
#[path = "dependencies/rows.rs"]
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
