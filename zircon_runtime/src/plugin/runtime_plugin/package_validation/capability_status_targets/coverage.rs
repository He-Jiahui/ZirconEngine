use crate::builtin::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

pub(super) fn validate_runtime_plugin_package_capability_status_target_coverage(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_mode: RuntimeTargetMode,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.supported_targets.contains(&target_mode) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` target mode {target_mode:?} must be covered by package supported_targets"
        ));
    }
}
