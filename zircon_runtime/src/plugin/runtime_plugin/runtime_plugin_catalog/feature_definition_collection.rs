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
    let mut definitions = HashMap::new();
    let mut diagnostics = Vec::new();
    let mut definition_order = Vec::new();
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
