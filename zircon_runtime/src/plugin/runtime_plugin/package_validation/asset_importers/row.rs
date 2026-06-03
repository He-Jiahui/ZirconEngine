use crate::asset::AssetImporterDescriptor;

use super::identity::validate_runtime_plugin_package_asset_importer_identity;
use super::required_capabilities::validate_runtime_plugin_package_asset_importer_required_capabilities;

pub(super) fn validate_runtime_plugin_package_asset_importer_row<'a>(
    package_id: &str,
    importer: &'a AssetImporterDescriptor,
    seen_ids: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_asset_importer_identity(
        package_id,
        importer,
        seen_ids,
        diagnostics,
    );
    validate_runtime_plugin_package_asset_importer_required_capabilities(importer, diagnostics);
}
