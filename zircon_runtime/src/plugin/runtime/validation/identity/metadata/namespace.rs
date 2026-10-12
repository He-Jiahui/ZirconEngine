use super::super::super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_asset_importer_id_namespace(
    importer_id: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_namespace("asset importer id", importer_id, diagnostics);
}
