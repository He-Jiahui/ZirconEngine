use crate::builtin::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

use super::super::coverage::validate_runtime_plugin_package_capability_status_target_coverage;

pub(super) fn validate_runtime_plugin_package_capability_status_target_row_coverage(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_mode: RuntimeTargetMode,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_target_coverage(
        package_manifest,
        capability,
        target_mode,
        diagnostics,
    );
}
