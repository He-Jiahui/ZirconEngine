use crate::core::framework::platform::RuntimeTargetMode;

use super::super::uniqueness::validate_runtime_plugin_package_capability_status_target_uniqueness;

pub(super) fn validate_runtime_plugin_package_capability_status_target_row_uniqueness(
    capability: &str,
    target_mode: RuntimeTargetMode,
    seen: &mut u8,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_target_uniqueness(
        capability,
        target_mode,
        seen,
        diagnostics,
    );
}
