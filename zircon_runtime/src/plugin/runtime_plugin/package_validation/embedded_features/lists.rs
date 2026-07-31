mod feature_extension;
mod optional;

use crate::plugin::PluginPackageManifest;

use self::{
    feature_extension::validate_runtime_plugin_package_feature_extension_list,
    optional::validate_runtime_plugin_package_optional_feature_list,
};
use super::super::projection::RuntimePluginPackageValidationProjection;

pub(super) fn validate_runtime_plugin_package_feature_lists(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_optional_feature_list(
        package_manifest,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_feature_extension_list(
        package_manifest,
        projection,
        diagnostics,
    );
}
