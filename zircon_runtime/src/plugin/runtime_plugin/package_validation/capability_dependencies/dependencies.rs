mod capability;
mod pairs;
mod row;
mod rows;

use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_dependency_rows(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_dependency_rows(package_manifest, diagnostics);
}
