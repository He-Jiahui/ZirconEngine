mod identity;
mod required_capabilities;
mod row;
mod rows;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_asset_importers(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_asset_importer_rows(package_manifest, diagnostics);
}
