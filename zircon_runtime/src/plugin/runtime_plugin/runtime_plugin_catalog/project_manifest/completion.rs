use crate::core::framework::project::ProjectPluginManifest;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_completion::complete_project_feature_selections;
use super::super::RuntimePluginRegistrationReport;
use super::selection_defaults::complete_project_selection_defaults;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn catalog_project_manifest(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
) -> ProjectPluginManifest {
    complete_project_manifest(
        registrations,
        projection,
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
    projection: &RuntimePluginCatalogProjection,
    manifest: &ProjectPluginManifest,
) -> ProjectPluginManifest {
    let mut completed = manifest.clone();
    complete_project_selection_defaults(registrations, projection, &mut completed);
    complete_project_feature_selections(projection, &mut completed);
    completed
}
