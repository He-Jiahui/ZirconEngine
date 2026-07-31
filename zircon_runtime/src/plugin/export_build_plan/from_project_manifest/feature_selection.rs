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
