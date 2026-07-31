pub(super) fn validate_runtime_plugin_package_asset_importer_required_capability_uniqueness(
    importer_id: &str,
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest asset importer `{importer_id}` required capability `{capability}` must be unique"
        ));
    }
}
