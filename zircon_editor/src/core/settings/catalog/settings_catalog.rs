use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::{SettingDefinition, SettingsKey, SettingsPresentation, SettingsRegistry};

/// Immutable setting-definition catalog compiled once when the authority is created.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsCatalog {
    definitions: Arc<[SettingDefinition]>,
    category_index: BTreeMap<Arc<str>, Arc<[SettingsKey]>>,
    subtree_index: BTreeMap<Arc<str>, Arc<[usize]>>,
}

impl SettingsCatalog {
    pub(in crate::core::settings) fn from_registry(registry: &SettingsRegistry) -> Self {
        let definitions = registry.definitions().cloned().collect::<Vec<_>>();
        let mut category_index = BTreeMap::<Arc<str>, Vec<SettingsKey>>::new();
        let mut subtree_index = BTreeMap::<Arc<str>, Vec<usize>>::new();
        for (definition_index, definition) in definitions.iter().enumerate() {
            let category_path = category_path_key(definition.presentation());
            for (separator, _) in category_path.match_indices('/') {
                append_subtree_index(
                    &mut subtree_index,
                    &category_path[..separator],
                    definition_index,
                );
            }
            append_subtree_index(&mut subtree_index, &category_path, definition_index);
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
            subtree_index: subtree_index
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

    /// Visits keys directly in a category and all of its descendants, in key order.
    pub(crate) fn keys_for_category_subtree(
        &self,
        category_path: &str,
    ) -> impl ExactSizeIterator<Item = &SettingsKey> + '_ {
        let indices: &[usize] = self
            .subtree_index
            .get(category_path)
            .map(|indices| indices.as_ref())
            .unwrap_or(&[]);
        indices
            .iter()
            .map(move |&index| &self.definitions[index].key)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

fn append_subtree_index(
    index: &mut BTreeMap<Arc<str>, Vec<usize>>,
    category_path: &str,
    definition_index: usize,
) {
    if let Some(indices) = index.get_mut(category_path) {
        indices.push(definition_index);
    } else {
        index.insert(Arc::from(category_path), vec![definition_index]);
    }
}

fn category_path_key(presentation: &SettingsPresentation) -> String {
    let segment_bytes = presentation.category_path().map(str::len).sum::<usize>();
    let separator_bytes = presentation.category_path().len().saturating_sub(1);
    let mut path = String::with_capacity(segment_bytes.saturating_add(separator_bytes));
    for (index, category) in presentation.category_path().enumerate() {
        if index != 0 {
            path.push('/');
        }
        path.push_str(category);
    }
    path
}

#[cfg(test)]
#[path = "settings_catalog/tests/optimization_batch_hy_editor608_tests.rs"]
mod optimization_batch_hy_editor608_tests;
