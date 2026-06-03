mod metadata;
mod uniqueness;

use crate::asset::AssetImporterDescriptor;

pub(super) fn validate_runtime_plugin_package_asset_importer_identity<'a>(
    package_id: &str,
    importer: &'a AssetImporterDescriptor,
    seen_ids: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    metadata::validate_runtime_plugin_package_asset_importer_metadata(
        package_id,
        importer,
        diagnostics,
    );
    uniqueness::validate_runtime_plugin_package_asset_importer_id_uniqueness(
        importer.id.as_str(),
        seen_ids,
        diagnostics,
    );
}
