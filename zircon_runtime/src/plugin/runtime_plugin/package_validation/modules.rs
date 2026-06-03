mod field;
mod row;
mod rows;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_modules(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_module_rows(package_manifest, diagnostics);
}
