mod state;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_ui_component_duplicate_row_state;
use self::uniqueness::validate_runtime_plugin_package_ui_component_id_uniqueness;

pub(super) fn validate_duplicate_ui_components(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_ui_component_duplicate_row_state();
    for component in &package_manifest.ui_components {
        validate_runtime_plugin_package_ui_component_id_uniqueness(
            component.component_id.as_str(),
            &mut seen,
            diagnostics,
        );
    }
}
