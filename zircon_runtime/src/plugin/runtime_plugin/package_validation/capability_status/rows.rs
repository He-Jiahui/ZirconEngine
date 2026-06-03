mod state;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_capability_status_row_state;
use super::row::validate_runtime_plugin_package_capability_status_row;

pub(super) fn validate_runtime_plugin_package_capability_status_rows(
    package_manifest: &PluginPackageManifest,
    owned_capabilities: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let mut seen_capabilities = new_runtime_plugin_package_capability_status_row_state();
    for status in &package_manifest.capability_statuses {
        validate_runtime_plugin_package_capability_status_row(
            package_manifest,
            status,
            owned_capabilities,
            &mut seen_capabilities,
            diagnostics,
        );
    }
}
