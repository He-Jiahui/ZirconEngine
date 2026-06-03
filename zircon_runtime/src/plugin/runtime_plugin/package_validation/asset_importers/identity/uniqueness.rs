pub(super) fn validate_runtime_plugin_package_asset_importer_id_uniqueness<'a>(
    importer_id: &'a str,
    seen_ids: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen_ids.contains(&importer_id) {
        diagnostics.push(format!(
            "runtime plugin package manifest asset importer id `{}` must be unique",
            importer_id
        ));
    } else {
        seen_ids.push(importer_id);
    }
}
