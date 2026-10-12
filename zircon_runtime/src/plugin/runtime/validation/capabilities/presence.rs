use crate::plugin::PluginPackageManifest;

/// 包必须显式声明至少一项包级能力；可选特性或模块的能力不会替代该包级声明。
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
