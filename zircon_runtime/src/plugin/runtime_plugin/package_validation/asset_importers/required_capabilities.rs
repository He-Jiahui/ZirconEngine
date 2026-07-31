mod namespace;
mod uniqueness;

use super::super::projection::RuntimePluginPackageValidationProjection;
use crate::asset::AssetImporterDescriptor;

use self::namespace::validate_runtime_plugin_package_asset_importer_required_capability_namespace;
use self::uniqueness::validate_runtime_plugin_package_asset_importer_required_capability_uniqueness;

pub(super) fn validate_runtime_plugin_package_asset_importer_required_capabilities(
    importer: &AssetImporterDescriptor,
    importer_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (capability_index, capability) in importer.required_capabilities.iter().enumerate() {
        validate_runtime_plugin_package_asset_importer_required_capability_namespace(
            capability,
            diagnostics,
        );
        validate_runtime_plugin_package_asset_importer_required_capability_uniqueness(
            &importer.id,
            capability,
            projection.asset_importer_capability_is_duplicate(importer_index, capability_index),
            diagnostics,
        );
    }
}
