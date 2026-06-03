mod ownership;
mod prefix;

use crate::plugin::PluginPackageManifest;

use self::ownership::validate_runtime_plugin_package_event_catalog_owner;
use self::prefix::new_runtime_plugin_package_event_catalog_owner_prefix;

pub(super) fn validate_event_catalog_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let expected_prefix =
        new_runtime_plugin_package_event_catalog_owner_prefix(package_manifest.id.as_str());
    for catalog in &package_manifest.event_catalogs {
        validate_runtime_plugin_package_event_catalog_owner(
            catalog.namespace.as_str(),
            package_manifest.id.as_str(),
            expected_prefix.as_str(),
            diagnostics,
        );
    }
}
