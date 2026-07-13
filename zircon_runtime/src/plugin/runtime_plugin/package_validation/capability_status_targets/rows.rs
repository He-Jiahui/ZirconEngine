mod state;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_capability_status_target_row_state;
use super::row::validate_runtime_plugin_package_capability_status_target_row;

pub(super) fn validate_runtime_plugin_package_capability_status_target_rows(
    package_manifest: &PluginPackageManifest,
    capability: &str,
    target_modes: &[RuntimeTargetMode],
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_capability_status_target_row_state();
    for target_mode in target_modes.iter().copied() {
        validate_runtime_plugin_package_capability_status_target_row(
            package_manifest,
            capability,
            target_mode,
            &mut seen,
            diagnostics,
        );
    }
}
