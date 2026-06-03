mod state;

use crate::plugin::PluginPackageManifest;

use self::state::new_runtime_plugin_package_asset_importer_row_state;
use super::row::validate_runtime_plugin_package_asset_importer_row;

pub(super) fn validate_runtime_plugin_package_asset_importer_rows(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen_ids = new_runtime_plugin_package_asset_importer_row_state();
    for importer in &package_manifest.asset_importers {
        validate_runtime_plugin_package_asset_importer_row(
            &package_manifest.id,
            importer,
            &mut seen_ids,
            diagnostics,
        );
    }
}
