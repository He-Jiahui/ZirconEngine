use std::collections::{HashMap, HashSet};

use super::super::feature_definitions::FeatureDefinition;
use super::super::package_feature_definitions::package_feature_definitions;
use super::super::RuntimePluginRegistrationReport;

fn package_feature_declaration_capacity(
    registrations: &[RuntimePluginRegistrationReport],
) -> usize {
    registrations.iter().fold(0, |capacity, registration| {
        capacity
            .saturating_add(registration.package_manifest.optional_features.len())
            .saturating_add(registration.package_manifest.feature_extensions.len())
    })
}

pub(super) fn merge_package_feature_definitions(
    registrations: &[RuntimePluginRegistrationReport],
    definitions: &mut HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    definition_order: &mut Vec<String>,
) -> HashSet<String> {
    let mut declared_feature_ids =
        HashSet::with_capacity(package_feature_declaration_capacity(registrations));
    for registration in registrations {
        for feature_definition in package_feature_definitions(&registration.package_manifest) {
            let key = feature_definition.key.clone();
            declared_feature_ids.insert(key.clone());
            if definitions
                .insert(key.clone(), feature_definition)
                .is_some()
            {
                diagnostics.push(format!(
                    "duplicate optional feature provider {} declared in plugin catalog",
                    key
                ));
            } else {
                definition_order.push(key);
            }
        }
    }
    declared_feature_ids
}

#[cfg(test)]
#[path = "package/capacity_tests.rs"]
mod capacity_tests;
