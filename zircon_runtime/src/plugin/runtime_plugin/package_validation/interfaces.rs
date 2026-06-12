mod dependencies;
mod exports;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_interfaces(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    exports::validate_runtime_plugin_package_provided_interfaces(package_manifest, diagnostics);
    dependencies::validate_runtime_plugin_package_dependency_interfaces(
        package_manifest,
        diagnostics,
    );
}
