mod array;
mod path;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_roots(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    array::validate_runtime_plugin_package_root_array(
        "asset_roots",
        &package_manifest.asset_roots,
        |index| projection.asset_root_is_duplicate(index),
        diagnostics,
    );
    array::validate_runtime_plugin_package_root_array(
        "content_roots",
        &package_manifest.content_roots,
        |index| projection.content_root_is_duplicate(index),
        diagnostics,
    );
}
