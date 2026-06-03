pub(super) type RuntimePluginPackageEventCatalogOwnerPrefix = String;

pub(super) fn new_runtime_plugin_package_event_catalog_owner_prefix(
    package_id: &str,
) -> RuntimePluginPackageEventCatalogOwnerPrefix {
    format!("{package_id}.")
}
