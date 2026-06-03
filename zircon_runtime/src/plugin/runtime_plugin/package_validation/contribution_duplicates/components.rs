mod state;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_component_duplicate_row_state;
use self::uniqueness::validate_runtime_plugin_package_component_type_uniqueness;

pub(super) fn validate_duplicate_components(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_component_duplicate_row_state();
    for component in &package_manifest.components {
        validate_runtime_plugin_package_component_type_uniqueness(
            component.type_id.as_str(),
            &mut seen,
            diagnostics,
        );
    }
}
