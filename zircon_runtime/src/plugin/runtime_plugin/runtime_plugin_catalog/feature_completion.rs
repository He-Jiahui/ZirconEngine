use std::collections::{HashMap, HashSet};

use crate::core::framework::project::ProjectPluginManifest;

mod owner_selection;

#[cfg(test)]
#[path = "feature_completion/capacity_tests.rs"]
mod capacity_tests;

use super::derived_projection::RuntimePluginCatalogProjection;
use owner_selection::{complete_external_provider_selection, complete_owner_feature_selection};

pub(super) fn complete_project_feature_selections(
    projection: &RuntimePluginCatalogProjection,
    completed: &mut ProjectPluginManifest,
) {
    let feature_definitions = projection.feature_definitions();
    let mut selected_package_ids = completed
        .selections
        .iter()
        .map(|selection| selection.id.clone())
        .collect::<HashSet<_>>();
    for selection in &mut completed.selections {
        let owner_id = selection.id.clone();
        let mut feature_indices = HashMap::with_capacity(selection.features.len());
        for (index, feature) in selection.features.iter().enumerate() {
            feature_indices.entry(feature.id.clone()).or_insert(index);
        }
        for feature_key in projection.definition_keys_for_owner(&owner_id) {
            let Some(feature_definition) = feature_definitions.definitions.get(feature_key) else {
                continue;
            };
            let feature = &feature_definition.manifest;
            complete_owner_feature_selection(
                selection,
                feature,
                feature_definition.external_provider_for_owner(),
                &mut feature_indices,
            );
        }
    }
    for feature_key in &feature_definitions.definition_order {
        let Some(feature_definition) = feature_definitions.definitions.get(feature_key) else {
            continue;
        };
        complete_external_provider_selection(
            completed,
            feature_definition,
            &mut selected_package_ids,
        );
    }
}
