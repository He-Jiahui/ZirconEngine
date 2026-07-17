use super::prefix::runtime_plugin_package_event_catalog_has_owner;

pub(super) fn validate_runtime_plugin_package_event_catalog_owner(
    event_catalog_namespace: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if !runtime_plugin_package_event_catalog_has_owner(package_id, event_catalog_namespace) {
        diagnostics.push(format!(
            "runtime plugin package manifest event catalog namespace `{event_catalog_namespace}` must be prefixed by package id `{package_id}`"
        ));
    }
}
