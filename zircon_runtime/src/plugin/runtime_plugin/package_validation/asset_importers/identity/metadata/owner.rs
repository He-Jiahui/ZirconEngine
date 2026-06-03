use crate::asset::AssetImporterDescriptor;

use super::super::super::super::validate_runtime_plugin_package_token;

pub(super) fn validate_runtime_plugin_package_asset_importer_owner(
    package_id: &str,
    importer: &AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_token(
        "asset importer plugin_id",
        &importer.plugin_id,
        diagnostics,
    );
    if importer.plugin_id != package_id {
        diagnostics.push(format!(
            "runtime plugin package manifest asset importer `{}` plugin_id `{}` must match package id `{}`",
            importer.id, importer.plugin_id, package_id
        ));
    }
}
