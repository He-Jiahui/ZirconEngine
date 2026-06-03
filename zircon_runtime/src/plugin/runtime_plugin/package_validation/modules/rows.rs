mod state;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_module_row_state;
use super::row::validate_runtime_plugin_package_module_row;

pub(super) fn validate_runtime_plugin_package_module_rows(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen_names = new_runtime_plugin_package_module_row_state();
    for module in &package_manifest.modules {
        validate_runtime_plugin_package_module_row(
            &package_manifest.id,
            package_manifest.supported_targets.as_slice(),
            module,
            &mut seen_names,
            diagnostics,
        );
    }
}
