mod coverage;
mod row;
mod rows;
mod uniqueness;

use crate::builtin::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_capability_status_targets(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_modes: &[RuntimeTargetMode],
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_package_capability_status_target_rows(
        package_manifest,
        capability,
        target_modes,
        diagnostics,
    );
}
