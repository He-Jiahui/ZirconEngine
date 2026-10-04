use crate::core::framework::project::ProjectPluginManifest;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_completion::complete_project_feature_selections;
use super::super::RuntimePluginRegistrationReport;
use super::selection_defaults::complete_project_selection_defaults;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn catalog_project_manifest(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
) -> ProjectPluginManifest {
    complete_project_manifest_owned(
        registrations,
        projection,
        ProjectPluginManifest {
            selections: registrations
                .iter()
                .filter(|registration| {
                    registration
                        .package_manifest
                        .package_role
                        .is_product_catalog_eligible()
                })
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
    complete_project_manifest_owned(registrations, projection, manifest.clone())
}

fn complete_project_manifest_owned(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    mut completed: ProjectPluginManifest,
) -> ProjectPluginManifest {
    complete_project_selection_defaults(registrations, projection, &mut completed);
    complete_project_feature_selections(projection, &mut completed);
    completed
}

#[cfg(test)]
#[path = "tests/completion_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/completion_product_catalog_tests.rs"]
mod product_catalog_tests;
