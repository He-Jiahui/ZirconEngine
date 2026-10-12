use crate::asset::AssetImporterDescriptor;

pub(super) fn validate_runtime_plugin_package_asset_importer_version(
    importer: &AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    if importer.importer_version == 0 {
        diagnostics.push(format!(
            "runtime plugin package manifest asset importer `{}` importer_version must be positive",
            importer.id
        ));
    }
}
