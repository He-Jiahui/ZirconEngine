mod namespace;
mod state;
mod uniqueness;

use crate::asset::AssetImporterDescriptor;

use self::namespace::validate_runtime_plugin_package_asset_importer_required_capability_namespace;
use self::state::new_runtime_plugin_package_asset_importer_required_capability_state;
use self::uniqueness::validate_runtime_plugin_package_asset_importer_required_capability_uniqueness;

pub(super) fn validate_runtime_plugin_package_asset_importer_required_capabilities(
    importer: &AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_package_asset_importer_required_capability_state();
    for capability in &importer.required_capabilities {
        validate_runtime_plugin_package_asset_importer_required_capability_namespace(
            capability,
            diagnostics,
        );
        validate_runtime_plugin_package_asset_importer_required_capability_uniqueness(
            &importer.id,
            capability,
            &mut seen,
            diagnostics,
        );
    }
}
