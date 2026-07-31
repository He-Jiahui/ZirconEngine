mod catalog_selections;
mod hydration;

use crate::core::framework::project::ProjectPluginManifest;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::RuntimePluginRegistrationReport;
use catalog_selections::add_missing_catalog_selections;
use hydration::hydrate_catalog_selection_defaults;

pub(super) fn complete_project_selection_defaults(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    completed: &mut ProjectPluginManifest,
) {
    add_missing_catalog_selections(registrations, completed);
    hydrate_catalog_selection_defaults(registrations, projection, completed);
}
