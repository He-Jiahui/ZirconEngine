mod array;
mod path;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_roots(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    array::validate_runtime_plugin_package_root_array(
        "asset_roots",
        &package_manifest.asset_roots,
        diagnostics,
    );
    array::validate_runtime_plugin_package_root_array(
        "content_roots",
        &package_manifest.content_roots,
        diagnostics,
    );
}
