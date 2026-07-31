mod metadata;
mod uniqueness;

use crate::asset::AssetImporterDescriptor;

pub(super) fn validate_runtime_plugin_package_asset_importer_identity(
    package_id: &str,
    importer: &AssetImporterDescriptor,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    metadata::validate_runtime_plugin_package_asset_importer_metadata(
        package_id,
        importer,
        diagnostics,
    );
    uniqueness::validate_runtime_plugin_package_asset_importer_id_uniqueness(
        importer.id.as_str(),
        is_duplicate,
        diagnostics,
    );
}
