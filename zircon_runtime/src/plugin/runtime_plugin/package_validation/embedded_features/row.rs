mod provider;
mod target_coverage;

use super::super::projection::{EmbeddedFeatureKind, RuntimePluginPackageValidationProjection};
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use self::{
    provider::validate_runtime_plugin_package_embedded_feature_provider,
    target_coverage::validate_runtime_plugin_package_embedded_feature_target_coverage,
};
use super::manifest::validate_runtime_plugin_package_embedded_feature_manifest;

/// 将同一嵌入行的通用特性约束、包内提供者身份和承载包目标覆盖集中累计到报告。
/// 类别及行号必须对应传入特性在原清单中的位置，否则共享投影会给出另一行的重复结果。
pub(super) fn validate_runtime_plugin_package_embedded_feature_row(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    kind: EmbeddedFeatureKind,
    feature_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_embedded_feature_manifest(
        feature,
        kind,
        feature_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_embedded_feature_provider(
        field_name,
        feature,
        package_manifest,
        projection.embedded_feature_provider_is_duplicate(kind, feature_index),
        diagnostics,
    );
    validate_runtime_plugin_package_embedded_feature_target_coverage(
        field_name,
        feature,
        package_manifest,
        diagnostics,
    );
}
