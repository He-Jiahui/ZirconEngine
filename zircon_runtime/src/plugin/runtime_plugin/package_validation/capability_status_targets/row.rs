mod coverage;
mod uniqueness;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

use self::{
    coverage::validate_runtime_plugin_package_capability_status_target_row_coverage,
    uniqueness::validate_runtime_plugin_package_capability_status_target_row_uniqueness,
};

pub(super) fn validate_runtime_plugin_package_capability_status_target_row(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_mode: RuntimeTargetMode,
    seen: &mut u8,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_target_row_uniqueness(
        capability,
        target_mode,
        seen,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_target_row_coverage(
        package_manifest,
        capability,
        target_mode,
        diagnostics,
    );
}
