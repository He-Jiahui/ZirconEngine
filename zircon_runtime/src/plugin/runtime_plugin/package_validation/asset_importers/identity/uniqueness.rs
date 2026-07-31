pub(super) fn validate_runtime_plugin_package_asset_importer_id_uniqueness(
    importer_id: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest asset importer id `{}` must be unique",
            importer_id
        ));
    }
}
