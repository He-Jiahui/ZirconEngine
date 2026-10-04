use super::super::super::projection::{
    EmbeddedFeatureKind, RuntimePluginPackageValidationProjection,
};
use crate::plugin::PluginPackageManifest;

use super::super::row::validate_runtime_plugin_package_embedded_feature_row;

/// 将可选特性的类别与原始行号传入通用行校验，以复用包投影中的特性内部重复信息。
/// 行号属于本列表，不能与扩展列表索引混用。
pub(super) fn validate_runtime_plugin_package_optional_feature_list(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (feature_index, feature) in package_manifest.optional_features.iter().enumerate() {
        validate_runtime_plugin_package_embedded_feature_row(
            "optional feature",
            feature,
            package_manifest,
            EmbeddedFeatureKind::Optional,
            feature_index,
            projection,
            diagnostics,
        );
    }
}
