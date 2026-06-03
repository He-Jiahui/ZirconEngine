use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_capability_presence(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if package_manifest.capabilities.is_empty() {
        diagnostics.push(
            "runtime plugin package manifest capabilities must declare at least one capability"
                .to_string(),
        );
    }
}
