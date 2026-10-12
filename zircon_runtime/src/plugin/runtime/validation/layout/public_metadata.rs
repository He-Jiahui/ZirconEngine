use crate::plugin::PluginPackageManifest;

use super::super::validate_runtime_plugin_package_field;
use super::description::validate_runtime_plugin_package_description;

pub(super) fn validate_runtime_plugin_package_public_metadata(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_field("category", &package_manifest.category, diagnostics);
    validate_runtime_plugin_package_description(package_manifest, diagnostics);
}
