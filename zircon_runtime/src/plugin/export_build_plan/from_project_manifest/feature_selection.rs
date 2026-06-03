use crate::{
    plugin::{ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection},
    RuntimeTargetMode,
};

use super::super::project_manifest_validation::{
    project_plugin_feature_id_is_valid, project_plugin_package_id_is_valid,
};

pub(super) fn feature_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    feature_id: &str,
) -> Option<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    manifest.selections.iter().find_map(|selection| {
        selection
            .features
            .iter()
            .find(|feature| feature.id == feature_id)
            .map(|feature| (selection, feature))
    })
}

pub(super) fn external_feature_selections(
    manifest: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> Vec<(&ProjectPluginSelection, &ProjectPluginFeatureSelection)> {
    manifest
        .selections
        .iter()
        .filter(move |selection| selection.enabled && selection.supports_target(target))
        .flat_map(move |selection| {
            selection
                .features
                .iter()
                .filter(move |feature| feature.enabled && feature.supports_target(target))
                .filter(move |feature| {
                    project_plugin_package_id_is_valid(&selection.id)
                        && project_plugin_feature_id_is_valid(&selection.id, &feature.id)
                        && feature
                            .external_provider_package_id(&selection.id)
                            .is_some_and(|provider_package_id| {
                                project_plugin_package_id_is_valid(provider_package_id)
                                    && manifest.selections.iter().any(|provider| {
                                        provider.id == provider_package_id
                                            && provider.enabled
                                            && provider.supports_target(target)
                                    })
                            })
                })
                .map(move |feature| (selection, feature))
        })
        .collect()
}

pub(super) fn external_feature_selection<'a>(
    manifest: &'a ProjectPluginManifest,
    feature_id: &str,
    target: RuntimeTargetMode,
) -> Option<(
    &'a ProjectPluginSelection,
    &'a ProjectPluginFeatureSelection,
)> {
    manifest.selections.iter().find_map(|selection| {
        if !selection.enabled || !selection.supports_target(target) {
            return None;
        }
        selection
            .features
            .iter()
            .find(|feature| {
                feature.id == feature_id
                    && feature.enabled
                    && feature.supports_target(target)
                    && project_plugin_package_id_is_valid(&selection.id)
                    && project_plugin_feature_id_is_valid(&selection.id, &feature.id)
                    && feature
                        .external_provider_package_id(&selection.id)
                        .is_some_and(|provider_package_id| {
                            project_plugin_package_id_is_valid(provider_package_id)
                                && manifest.selections.iter().any(|provider| {
                                    provider.id == provider_package_id
                                        && provider.enabled
                                        && provider.supports_target(target)
                                })
                        })
            })
            .map(|feature| (selection, feature))
    })
}
