use std::collections::HashSet;

use crate::core::framework::project::ProjectPluginManifest;

use super::super::super::RuntimePluginRegistrationReport;

pub(super) fn add_missing_catalog_selections(
    registrations: &[RuntimePluginRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    let mut selected_package_ids = completed
        .selections
        .iter()
        .map(|selection| selection.id.clone())
        .collect::<HashSet<_>>();
    for registration in registrations {
        if selected_package_ids.contains(&registration.project_selection.id) {
            continue;
        }
        let mut selection = registration.project_selection.clone();
        selection.enabled = false;
        selected_package_ids.insert(selection.id.clone());
        completed.selections.push(selection);
    }
}
