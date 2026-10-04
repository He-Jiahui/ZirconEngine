use std::collections::HashSet;

use crate::core::framework::project::ProjectPluginManifest;

use super::super::super::RuntimePluginRegistrationReport;

pub(super) fn add_missing_catalog_selections(
    registrations: &[RuntimePluginRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    let selection_capacity = completed
        .selections
        .len()
        .saturating_add(registrations.len());
    let mut selected_package_ids = HashSet::with_capacity(selection_capacity);
    for selection in &completed.selections {
        selected_package_ids.insert(selection.id.clone());
    }
    completed.selections.reserve(registrations.len());
    for registration in registrations {
        if !registration
            .package_manifest
            .package_role
            .is_product_catalog_eligible()
        {
            continue;
        }
        if selected_package_ids.contains(&registration.project_selection.id) {
            continue;
        }
        let mut selection = registration.project_selection.clone();
        selection.enabled = false;
        selected_package_ids.insert(selection.id.clone());
        completed.selections.push(selection);
    }
}

#[cfg(test)]
#[path = "tests/catalog_selections.rs"]
mod tests;
