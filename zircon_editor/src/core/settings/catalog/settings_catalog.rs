use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::{SettingDefinition, SettingsKey, SettingsRegistry};

/// Immutable setting-definition catalog compiled once when the authority is created.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsCatalog {
    definitions: Arc<[SettingDefinition]>,
    category_index: BTreeMap<Arc<str>, Arc<[SettingsKey]>>,
}

impl SettingsCatalog {
    pub(in crate::core::settings) fn from_registry(registry: &SettingsRegistry) -> Self {
        let definitions = registry.definitions().cloned().collect::<Vec<_>>();
        let mut category_index = BTreeMap::<Arc<str>, Vec<SettingsKey>>::new();
        for definition in &definitions {
            let category_path = definition
                .presentation()
                .category_path()
                .collect::<Vec<_>>()
                .join("/");
            category_index
                .entry(Arc::from(category_path))
                .or_default()
                .push(definition.key.clone());
        }
        Self {
            definitions: definitions.into(),
            category_index: category_index
                .into_iter()
                .map(|(path, keys)| (path, keys.into()))
                .collect(),
        }
    }

    pub fn definitions(&self) -> &[SettingDefinition] {
        &self.definitions
    }

    pub fn definition(&self, key: &SettingsKey) -> Option<&SettingDefinition> {
        self.definitions
            .binary_search_by(|definition| definition.key.cmp(key))
            .ok()
            .map(|index| &self.definitions[index])
    }

    /// Returns the canonical keys assigned directly to one locale-neutral category path.
    pub fn keys_for_category_path(&self, category_path: &str) -> &[SettingsKey] {
        self.category_index
            .get(category_path)
            .map(|keys| keys.as_ref())
            .unwrap_or(&[])
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
