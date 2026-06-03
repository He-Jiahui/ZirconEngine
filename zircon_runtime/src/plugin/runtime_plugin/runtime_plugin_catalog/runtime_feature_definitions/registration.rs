use std::collections::{HashMap, HashSet};

use super::super::feature_definitions::FeatureDefinition;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::conflict::append_package_registration_conflict;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_feature_registration(
    registration: &RuntimePluginFeatureRegistrationReport,
    definitions: &mut HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    definition_order: &mut Vec<String>,
    declared_feature_ids: &HashSet<String>,
    registered_feature_ids: &mut HashSet<String>,
) {
    let feature_definition = FeatureDefinition::new(
        registration.manifest.clone(),
        registration.provider_package_id_or_owner().to_string(),
    );
    let key = feature_definition.key.clone();
    if !registered_feature_ids.insert(key.clone()) {
        diagnostics.push(format!(
            "duplicate optional feature id {} registered at runtime (provider {})",
            registration.manifest.id, feature_definition.provider_package_id
        ));
        return;
    }
    if declared_feature_ids.contains(&key) {
        append_package_registration_conflict(registration, definitions, diagnostics, &key);
        return;
    }
    if definitions
        .insert(key.clone(), feature_definition)
        .is_some()
    {
        diagnostics.push(format!(
            "duplicate optional feature provider {} declared or registered in plugin catalog",
            key
        ));
    } else {
        definition_order.push(key);
    }
}
