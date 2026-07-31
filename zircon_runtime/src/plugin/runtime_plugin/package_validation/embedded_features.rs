mod kind;
mod lists;
mod manifest;
mod row;

use super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_embedded_features(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    kind::validate_runtime_plugin_package_feature_kind(package_manifest, diagnostics);
    lists::validate_runtime_plugin_package_feature_lists(package_manifest, projection, diagnostics);
}
