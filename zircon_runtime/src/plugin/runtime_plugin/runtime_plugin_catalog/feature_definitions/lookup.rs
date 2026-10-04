use crate::core::framework::project::ProjectPluginFeatureSelection;

use super::super::feature_support::plugin_ids_match;
use super::key::feature_definition_key;
use super::{FeatureDefinition, FeatureDefinitionMap};

impl FeatureDefinitionMap {
    // 先用显式 provider 或 owner 组成精确键，再按定义顺序允许 canonical alias 匹配；不猜测未指定的外部 provider。
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn definition_for_selection(
        &self,
        owner_plugin_id: &str,
        feature: &ProjectPluginFeatureSelection,
    ) -> Option<&FeatureDefinition> {
        let requested_provider = feature
            .provider_package_id
            .as_deref()
            .unwrap_or(owner_plugin_id);
        let preferred_key = feature_definition_key(&feature.id, requested_provider);
        self.definitions.get(&preferred_key).or_else(|| {
            self.definition_order.iter().find_map(|key| {
                self.definitions.get(key).filter(|definition| {
                    definition.manifest.id == feature.id
                        && plugin_ids_match(&definition.provider_package_id, requested_provider)
                })
            })
        })
    }
}

#[cfg(test)]
#[path = "tests/lookup.rs"]
mod tests;
