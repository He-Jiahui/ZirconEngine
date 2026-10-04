mod package;

use std::collections::HashMap;

use self::package::{merge_package_feature_definitions, package_feature_declaration_capacity};
use super::feature_definitions::FeatureDefinitionMap;
use super::runtime_feature_definitions::{
    merge_runtime_feature_definitions, product_catalog_feature_registration_count,
};
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

// 先收集包清单声明，再并入具体 Runtime feature 登记；相同 feature@provider 键用于冲突检查与选择查找。
pub(super) fn feature_definition_map(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
) -> FeatureDefinitionMap {
    let definition_capacity = package_feature_declaration_capacity(registrations).saturating_add(
        product_catalog_feature_registration_count(feature_registrations),
    );
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
#[path = "feature_definition_collection/tests/capacity_tests.rs"]
mod capacity_tests;
