use std::collections::{HashMap, HashSet};

use super::super::feature_definitions::FeatureDefinition;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::registration::merge_runtime_feature_registration;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn product_catalog_feature_registration_count(
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
) -> usize {
    feature_registrations
        .iter()
        .filter(|registration| registration.is_product_catalog_eligible())
        .count()
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_feature_definitions(
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    definitions: &mut HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    definition_order: &mut Vec<String>,
    declared_feature_ids: &HashSet<String>,
) {
    let mut registered_feature_ids = HashSet::with_capacity(
        product_catalog_feature_registration_count(feature_registrations),
    );
    for registration in feature_registrations {
        // Carrier reports remain inspectable, but cannot define product catalog features.
        if !registration.is_product_catalog_eligible() {
            continue;
        }
        merge_runtime_feature_registration(
            registration,
            definitions,
            diagnostics,
            definition_order,
            declared_feature_ids,
            &mut registered_feature_ids,
        );
    }
}

#[cfg(test)]
#[path = "merge/tests/capacity_tests.rs"]
mod capacity_tests;
