mod state;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_capability_row_state;
use super::row::validate_runtime_plugin_package_capability_row;

pub(super) fn validate_runtime_plugin_package_capability_rows(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_capability_row_state();
    for capability in &package_manifest.capabilities {
        validate_runtime_plugin_package_capability_row(capability, &mut seen, diagnostics);
    }
}
