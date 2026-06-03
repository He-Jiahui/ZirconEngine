mod fields;
mod presence;
mod shape;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_coordinates(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    fields::validate_runtime_plugin_package_coordinate_fields(package_manifest, diagnostics);
}
