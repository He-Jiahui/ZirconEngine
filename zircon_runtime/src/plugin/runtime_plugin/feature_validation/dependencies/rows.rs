mod primary;

use crate::plugin::PluginFeatureBundleManifest;

use self::primary::FeaturePrimaryDependencyRows;
use super::super::projection::RuntimePluginFeatureValidationProjection;
use super::row;

// 行检查会累积诊断；必须遍历整个依赖集合后，再裁定主依赖的整体数量。
pub(super) fn validate_runtime_plugin_feature_dependency_rows(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    let mut primary = FeaturePrimaryDependencyRows::default();
    for (index, dependency) in feature.dependencies.iter().enumerate() {
        row::validate_runtime_plugin_feature_dependency_row(dependency, diagnostics);
        super::pairs::validate_runtime_plugin_feature_dependency_pair(
            dependency,
            projection.dependency_is_duplicate(index),
            diagnostics,
        );
        primary.validate(dependency, &feature.owner_plugin_id, diagnostics);
    }
    primary.validate_count(diagnostics);
}
