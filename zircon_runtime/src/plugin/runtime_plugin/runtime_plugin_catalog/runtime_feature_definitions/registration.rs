use std::collections::{HashMap, HashSet};

use super::super::feature_definitions::FeatureDefinition;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::conflict::append_package_registration_conflict;

// 先按特性与提供者键拒绝重复运行时登记，避免为该重复项克隆清单；包声明冲突在构造定义后另行诊断。
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_feature_registration(
    registration: &RuntimePluginFeatureRegistrationReport,
    definitions: &mut HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    definition_order: &mut Vec<String>,
    declared_feature_ids: &HashSet<String>,
    registered_feature_ids: &mut HashSet<String>,
) {
    let provider_package_id = registration.provider_package_id_or_owner();
    let key = FeatureDefinition::key(&registration.manifest.id, provider_package_id);
    if registered_feature_ids.contains(&key) {
        diagnostics.push(format!(
            "duplicate optional feature id {} registered at runtime (provider {})",
            registration.manifest.id, provider_package_id
        ));
        return;
    }
    registered_feature_ids.insert(key.clone());
    let feature_definition = FeatureDefinition::new_with_key(
        key.clone(),
        registration.manifest.clone(),
        provider_package_id.to_owned(),
    );
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

#[cfg(test)]
#[path = "tests/registration_optimization_tests.rs"]
mod optimization_tests;
