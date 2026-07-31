use crate::plugin::PluginPackageManifest;

use super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_asset_importer_row;

pub(super) fn validate_runtime_plugin_package_asset_importer_rows(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (importer_index, importer) in package_manifest.asset_importers.iter().enumerate() {
        validate_runtime_plugin_package_asset_importer_row(
            &package_manifest.id,
            importer,
            importer_index,
            projection,
            diagnostics,
        );
    }
}
