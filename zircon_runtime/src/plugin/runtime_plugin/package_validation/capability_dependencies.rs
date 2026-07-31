mod capabilities;
mod dependencies;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capabilities(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    capabilities::validate_runtime_plugin_package_capability_rows(
        package_manifest,
        projection,
        diagnostics,
    );
}

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_dependencies(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    dependencies::validate_runtime_plugin_package_dependency_rows(
        package_manifest,
        projection,
        diagnostics,
    );
}
