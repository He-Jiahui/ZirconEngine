mod presence;
mod row;
mod rows;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_capability_rows(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    presence::validate_runtime_plugin_package_capability_presence(package_manifest, diagnostics);
    rows::validate_runtime_plugin_package_capability_rows(package_manifest, diagnostics);
}
