mod namespace;
mod owner;
mod version;

use crate::asset::AssetImporterDescriptor;

use self::namespace::validate_runtime_plugin_package_asset_importer_id_namespace;
use self::owner::validate_runtime_plugin_package_asset_importer_owner;
use self::version::validate_runtime_plugin_package_asset_importer_version;

pub(super) fn validate_runtime_plugin_package_asset_importer_metadata(
    package_id: &str,
    importer: &AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_asset_importer_id_namespace(&importer.id, diagnostics);
    validate_runtime_plugin_package_asset_importer_owner(package_id, importer, diagnostics);
    validate_runtime_plugin_package_asset_importer_version(importer, diagnostics);
}
