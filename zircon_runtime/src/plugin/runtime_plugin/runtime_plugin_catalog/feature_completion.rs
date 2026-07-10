use crate::plugin::ProjectPluginManifest;

mod owner_selection;

use super::feature_definition_collection::feature_definition_map;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use owner_selection::{complete_external_provider_selection, complete_owner_feature_selection};

pub(super) fn complete_project_feature_selections(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    completed: &mut ProjectPluginManifest,
) {
    let feature_definitions = feature_definition_map(registrations, feature_registrations);
    for selection in &mut completed.selections {
        let owner_id = selection.id.clone();
        for feature_key in &feature_definitions.definition_order {
            let Some(feature_definition) = feature_definitions.definitions.get(feature_key) else {
                continue;
            };
            let feature = &feature_definition.manifest;
            if feature.owner_plugin_id != owner_id {
                continue;
            }
            complete_owner_feature_selection(
                selection,
                feature,
                feature_definition.external_provider_for_owner(),
            );
        }
    }
    for feature_key in &feature_definitions.definition_order {
        let Some(feature_definition) = feature_definitions.definitions.get(feature_key) else {
            continue;
        };
        complete_external_provider_selection(completed, feature_definition);
    }
}
