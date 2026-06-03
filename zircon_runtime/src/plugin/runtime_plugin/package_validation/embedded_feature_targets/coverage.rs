use crate::{plugin::PluginPackageManifest, RuntimeTargetMode};

pub(super) fn validate_runtime_plugin_package_feature_target_coverage(
    field_name: &str,
    feature_id: &str,
    module_name: &str,
    package_manifest: &PluginPackageManifest,
    target_mode: RuntimeTargetMode,
    diagnostics: &mut Vec<String>,
) {
    if !package_manifest.supported_targets.contains(&target_mode) {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{feature_id}` module `{module_name}` target mode {target_mode:?} must be covered by package supported_targets",
        ));
    }
}
