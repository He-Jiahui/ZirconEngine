mod catalog_selections;
mod hydration;

use crate::plugin::ProjectPluginManifest;

use super::super::RuntimePluginRegistrationReport;
use catalog_selections::add_missing_catalog_selections;
use hydration::hydrate_catalog_selection_defaults;

pub(super) fn complete_project_selection_defaults(
    registrations: &[RuntimePluginRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    add_missing_catalog_selections(registrations, completed);
    hydrate_catalog_selection_defaults(registrations, completed);
}
