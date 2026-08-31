mod package;

use std::collections::HashMap;

use self::package::merge_package_feature_definitions;
use super::feature_definitions::FeatureDefinitionMap;
use super::runtime_feature_definitions::merge_runtime_feature_definitions;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

pub(super) fn feature_definition_map(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
) -> FeatureDefinitionMap {
    let definition_capacity = registrations
        .len()
        .saturating_add(feature_registrations.len());
    let mut definitions = HashMap::with_capacity(definition_capacity);
    let mut diagnostics = Vec::new();
    let mut definition_order = Vec::with_capacity(definition_capacity);
    let declared_feature_ids = merge_package_feature_definitions(
        registrations,
        &mut definitions,
        &mut diagnostics,
        &mut definition_order,
    );
    merge_runtime_feature_definitions(
        feature_registrations,
        &mut definitions,
        &mut diagnostics,
        &mut definition_order,
        &declared_feature_ids,
    );
    FeatureDefinitionMap {
        definitions,
        diagnostics,
        definition_order,
    }
}

#[cfg(test)]
#[path = "feature_definition_collection/capacity_tests.rs"]
mod capacity_tests;
