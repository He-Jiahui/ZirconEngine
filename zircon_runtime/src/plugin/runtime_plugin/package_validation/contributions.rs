mod groups;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_contributions(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    groups::validate_runtime_plugin_package_contribution_groups(package_manifest, diagnostics);
}
