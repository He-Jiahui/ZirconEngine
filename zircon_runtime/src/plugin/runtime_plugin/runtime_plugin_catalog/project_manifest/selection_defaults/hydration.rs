use crate::plugin::ProjectPluginManifest;

use super::super::super::RuntimePluginRegistrationReport;
use super::super::lookup::project_selection_for_package;

pub(super) fn hydrate_catalog_selection_defaults(
    registrations: &[RuntimePluginRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    for selection in &mut completed.selections {
        if let Some(catalog_selection) = project_selection_for_package(registrations, &selection.id)
        {
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
