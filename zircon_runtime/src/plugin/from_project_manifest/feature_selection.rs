//! 将特征查找集中到同一代清单投影；建计划时本地特征和外部 provider 使用相同的选择视图。
use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};

use super::super::project_manifest_validation::ProjectPluginManifestValidationProjection;

pub(super) fn feature_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
    feature_id: &str,
) -> Option<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    projection.feature_selection(manifest, feature_id)
}

/// 仅从已刷新 projection 取符合目标与 provider 资格的特征，供构建链接与包选择使用。
pub(super) fn external_feature_selections<'a>(
    manifest: &'a ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
) -> Vec<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    projection.external_feature_selections(manifest)
}

pub(super) fn external_feature_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
    feature_id: &str,
) -> Option<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    projection.external_feature_selection(manifest, feature_id)
}
