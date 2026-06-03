mod state;
mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_event_catalog_duplicate_row_state;
use self::uniqueness::validate_runtime_plugin_package_event_catalog_namespace_uniqueness;

pub(super) fn validate_duplicate_event_catalogs(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_event_catalog_duplicate_row_state();
    for catalog in &package_manifest.event_catalogs {
        validate_runtime_plugin_package_event_catalog_namespace_uniqueness(
            catalog.namespace.as_str(),
            &mut seen,
            diagnostics,
        );
    }
}
