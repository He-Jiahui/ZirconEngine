use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::RuntimeExtensionRegistry;

use super::super::extension_merge::merge_feature_extensions;
use super::super::feature_registration_match::feature_registration_matches_project_selection;
use super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::RuntimePluginFeatureRegistrationReport;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_available_feature_extensions(
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    completed: &ProjectPluginManifest,
    feature_report: &RuntimePluginFeatureDependencyReport,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for feature_id in &feature_report.available_features {
        if let Some(registration) = feature_registrations.iter().find(|registration| {
            registration.manifest.id == *feature_id
                && feature_registration_matches_project_selection(
                    registration,
                    completed,
                    feature_id,
                )
        }) {
            merge_feature_extensions(registration, registry, diagnostics, fatal_diagnostics);
        }
    }
}
