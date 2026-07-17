mod ownership;
mod prefix;

use crate::plugin::PluginPackageManifest;

use self::ownership::validate_runtime_plugin_package_event_catalog_owner;

pub(super) fn validate_event_catalog_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    for catalog in &package_manifest.event_catalogs {
        validate_runtime_plugin_package_event_catalog_owner(
            catalog.namespace.as_str(),
            package_manifest.id.as_str(),
            diagnostics,
        );
    }
}
