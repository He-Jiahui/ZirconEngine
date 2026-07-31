use crate::plugin::RuntimeExtensionRegistry;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::extension_merge::merge_feature_extensions;
use super::super::feature_registration_match::{
    feature_registration_matches_project_selection, ProjectFeatureProviderLookup,
};
use super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::RuntimePluginFeatureRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_available_feature_extensions(
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    selected_providers: &ProjectFeatureProviderLookup<'_>,
    feature_report: &RuntimePluginFeatureDependencyReport,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for feature_id in &feature_report.available_features {
        if let Some(registration) = projection
            .feature_registration_indices(feature_id)
            .iter()
            .map(|index| &feature_registrations[*index])
            .find(|registration| {
                feature_registration_matches_project_selection(
                    registration,
                    selected_providers,
                    feature_id,
                )
            })
        {
            merge_feature_extensions(registration, registry, diagnostics, fatal_diagnostics);
        }
    }
}
