mod dependencies;
mod exports;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_interfaces(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    exports::validate_runtime_plugin_package_provided_interfaces(
        package_manifest,
        projection,
        diagnostics,
    );
    dependencies::validate_runtime_plugin_package_dependency_interfaces(
        package_manifest,
        projection,
        diagnostics,
    );
}
