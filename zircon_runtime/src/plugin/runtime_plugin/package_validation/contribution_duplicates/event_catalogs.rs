mod uniqueness;

use crate::plugin::PluginPackageManifest;

use self::uniqueness::validate_runtime_plugin_package_event_catalog_namespace_uniqueness;
use super::super::projection::RuntimePluginPackageValidationProjection;

/// 校验包内事件目录的命名空间身份；投影必须与当前清单的目录行序一致。
/// 是否属于此包是另一项校验，重复条目仍需继续接受归属检查。
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
