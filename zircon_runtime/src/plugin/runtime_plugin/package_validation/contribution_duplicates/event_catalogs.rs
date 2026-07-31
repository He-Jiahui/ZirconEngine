mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_event_catalog_namespace_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

pub(super) fn validate_duplicate_event_catalogs(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, catalog) in package_manifest.event_catalogs.iter().enumerate() {
        validate_runtime_plugin_package_event_catalog_namespace_uniqueness(
            catalog.namespace.as_str(),
            projection.event_catalog_namespace_is_duplicate(index),
            diagnostics,
        );
    }
}
