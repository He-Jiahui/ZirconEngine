use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_description(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.description.is_empty()
        && package_manifest.description.trim() != package_manifest.description
    {
        diagnostics.push(format!(
            "runtime plugin package manifest description `{}` must be trimmed when present",
            package_manifest.description
        ));
    }
}
