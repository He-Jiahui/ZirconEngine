use crate::plugin::PluginPackageManifest;

pub(super) fn collect_runtime_plugin_package_owned_capabilities(
    package_manifest: &PluginPackageManifest,
) -> Vec<&str> {
    let mut owned_capabilities = package_manifest
        .capabilities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    owned_capabilities.extend(
        package_manifest
            .optional_features
            .iter()
            .flat_map(|feature| feature.capabilities.iter().map(String::as_str)),
    );
    owned_capabilities
}
