use crate::core::framework::project::ProjectPluginManifest;

use super::super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::super::RuntimePluginRegistrationReport;

pub(super) fn hydrate_catalog_selection_defaults(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    completed: &mut ProjectPluginManifest,
) {
    for selection in &mut completed.selections {
        if let Some(registration_index) = projection.registration_index_for_package(&selection.id) {
            let catalog_selection = &registrations[registration_index].project_selection;
            if selection.runtime_crate.is_none() {
                selection.runtime_crate = catalog_selection.runtime_crate.clone();
            }
            if selection.editor_crate.is_none() {
                selection.editor_crate = catalog_selection.editor_crate.clone();
            }
            if selection.target_modes.is_empty() {
                selection.target_modes = catalog_selection.target_modes.clone();
            }
        }
    }
}
