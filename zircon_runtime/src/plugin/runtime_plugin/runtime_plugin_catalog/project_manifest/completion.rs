use crate::plugin::ProjectPluginManifest;

use super::super::feature_completion::complete_project_feature_selections;
use super::super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use super::selection_defaults::complete_project_selection_defaults;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn catalog_project_manifest(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
) -> ProjectPluginManifest {
    complete_project_manifest(
        registrations,
        feature_registrations,
        &ProjectPluginManifest {
            selections: registrations
                .iter()
                .map(|registration| registration.project_selection.clone())
                .collect(),
        },
    )
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_project_manifest(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    manifest: &ProjectPluginManifest,
) -> ProjectPluginManifest {
    let mut completed = manifest.clone();
    complete_project_selection_defaults(registrations, &mut completed);
    complete_project_feature_selections(registrations, feature_registrations, &mut completed);
    completed
}
