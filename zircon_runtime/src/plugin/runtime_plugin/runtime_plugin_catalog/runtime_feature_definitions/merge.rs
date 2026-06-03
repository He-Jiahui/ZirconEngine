use std::collections::{HashMap, HashSet};

use super::super::feature_definitions::FeatureDefinition;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::registration::merge_runtime_feature_registration;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_feature_definitions(
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    definitions: &mut HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    definition_order: &mut Vec<String>,
    declared_feature_ids: &HashSet<String>,
) {
    let mut registered_feature_ids = HashSet::new();
    for registration in feature_registrations {
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
