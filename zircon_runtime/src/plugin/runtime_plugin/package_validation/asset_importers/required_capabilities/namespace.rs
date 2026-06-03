use super::super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_asset_importer_required_capability_namespace(
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_namespace(
        "asset importer required capability",
        capability,
        diagnostics,
    );
}
