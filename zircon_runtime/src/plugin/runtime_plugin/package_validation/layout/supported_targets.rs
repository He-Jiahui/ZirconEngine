mod state;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_supported_target_state;

pub(super) fn validate_runtime_plugin_package_supported_targets(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_supported_target_state();
    for target_mode in package_manifest.supported_targets.iter().copied() {
        uniqueness::validate_runtime_plugin_package_supported_target_uniqueness(
            target_mode,
            &mut seen,
            diagnostics,
        );
    }
}
