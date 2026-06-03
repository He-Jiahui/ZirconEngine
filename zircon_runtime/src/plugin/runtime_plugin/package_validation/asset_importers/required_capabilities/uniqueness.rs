pub(super) fn validate_runtime_plugin_package_asset_importer_required_capability_uniqueness<'a>(
    importer_id: &str,
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&capability) {
        diagnostics.push(format!(
            "runtime plugin package manifest asset importer `{importer_id}` required capability `{capability}` must be unique"
        ));
    } else {
        seen.push(capability);
    }
}
