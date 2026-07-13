use crate::core::framework::project::ProjectPluginManifest;

use super::super::super::RuntimePluginRegistrationReport;

pub(super) fn add_missing_catalog_selections(
    registrations: &[RuntimePluginRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    for registration in registrations {
        if completed
            .selections
            .iter()
            .any(|selection| selection.id == registration.project_selection.id)
        {
            continue;
        }
        let mut selection = registration.project_selection.clone();
        selection.enabled = false;
        completed.selections.push(selection);
    }
}
