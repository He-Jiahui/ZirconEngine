mod description;
mod public_metadata;
mod supported_platforms;
mod supported_targets;

use crate::plugin::PluginPackageManifest;

use super::coordinates::validate_runtime_plugin_package_coordinates;
use super::roots::validate_runtime_plugin_package_roots;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_layout(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    public_metadata::validate_runtime_plugin_package_public_metadata(package_manifest, diagnostics);
    validate_runtime_plugin_package_coordinates(package_manifest, diagnostics);
    supported_targets::validate_runtime_plugin_package_supported_targets(
        package_manifest,
        diagnostics,
    );
    supported_platforms::validate_runtime_plugin_package_supported_platforms(
        package_manifest,
        diagnostics,
    );
    validate_runtime_plugin_package_roots(package_manifest, diagnostics);
}
