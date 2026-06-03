use crate::plugin::ProjectPluginFeatureSelection;

use super::key::feature_definition_key;
use super::{FeatureDefinition, FeatureDefinitionMap};

impl FeatureDefinitionMap {
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
        if let Some(definition) = self.definitions.get(&preferred_key) {
            return Some(definition);
        }
        if feature.provider_package_id.is_some() {
            return None;
        }
        self.definitions
            .values()
            .filter(|definition| definition.manifest.id == feature.id)
            .single()
    }
}

trait SingleDefinition<'a> {
    fn single(self) -> Option<&'a FeatureDefinition>;
}

impl<'a, I> SingleDefinition<'a> for I
where
    I: Iterator<Item = &'a FeatureDefinition>,
{
    fn single(mut self) -> Option<&'a FeatureDefinition> {
        let value = self.next()?;
        if self.next().is_some() {
            return None;
        }
        Some(value)
    }
}
