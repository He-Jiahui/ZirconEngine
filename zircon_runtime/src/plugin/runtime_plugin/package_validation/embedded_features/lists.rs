mod feature_extension;
mod optional;

use crate::plugin::PluginPackageManifest;

use self::{
    feature_extension::validate_runtime_plugin_package_feature_extension_list,
    optional::validate_runtime_plugin_package_optional_feature_list,
};
use super::super::projection::RuntimePluginPackageValidationProjection;

/// 按可选列表、扩展列表的固定顺序检查所有行，包括包类型校验已经判为非法的列表。
/// 各列表的原始行序必须与同一清单构建的共享投影一致。
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
