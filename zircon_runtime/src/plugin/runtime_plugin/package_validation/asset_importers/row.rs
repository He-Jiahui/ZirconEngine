use super::super::projection::RuntimePluginPackageValidationProjection;
use crate::asset::AssetImporterDescriptor;

use super::identity::validate_runtime_plugin_package_asset_importer_identity;
use super::required_capabilities::validate_runtime_plugin_package_asset_importer_required_capabilities;

pub(super) fn validate_runtime_plugin_package_asset_importer_row(
    package_id: &str,
    importer: &AssetImporterDescriptor,
    importer_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_asset_importer_identity(
        package_id,
        importer,
        projection.asset_importer_id_is_duplicate(importer_index),
        diagnostics,
    );
    validate_runtime_plugin_package_asset_importer_required_capabilities(
        importer,
        importer_index,
        projection,
        diagnostics,
    );
}
