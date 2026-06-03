mod identity;
mod note;
mod owned_capabilities;
mod row;
mod rows;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capability_statuses(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let owned_capabilities =
        owned_capabilities::collect_runtime_plugin_package_owned_capabilities(package_manifest);
    rows::validate_runtime_plugin_package_capability_status_rows(
        package_manifest,
        &owned_capabilities,
        diagnostics,
    );
}
