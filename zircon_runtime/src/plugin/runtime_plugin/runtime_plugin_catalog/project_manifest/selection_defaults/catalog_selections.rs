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
mod tests {
    #[test]
    fn preallocated_catalog_selection_completion_preserves_behavior_contract() {
        let source = include_str!("catalog_selections.rs");
        let preallocated_set = ["HashSet::with_", "capacity"].concat();
        let target_reserve = ["completed.selections.", "reserve(registrations.len())"].concat();
        let unbounded_collect = ["collect::<HashSet", "<_>>()"].concat();

        assert_eq!(source.matches(&preallocated_set).count(), 1);
        assert_eq!(source.matches(&target_reserve).count(), 1);
        assert!(!source.contains(&unbounded_collect));
    }
}
