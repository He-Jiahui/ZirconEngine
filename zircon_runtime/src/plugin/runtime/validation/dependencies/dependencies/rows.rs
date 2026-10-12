use crate::plugin::PluginPackageManifest;

use super::super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_dependency_row;

pub(super) fn validate_runtime_plugin_package_dependency_rows(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, dependency) in package_manifest.dependencies.iter().enumerate() {
        validate_runtime_plugin_package_dependency_row(
            dependency,
            projection.dependency_capability_is_duplicate(index),
            diagnostics,
        );
    }
}
