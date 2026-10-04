use std::collections::HashSet;

use crate::asset::{AssetUri, AssetUuid};

use super::asset_registry_index::source_locator;
use super::{AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// Builds a source-deletion candidate after topology admission has succeeded.
    /// 先按 source locator 收集根资产及其子资产，再克隆索引删除整组，调用方随后发布 generation。
    pub(crate) fn prepare_source_deletion_generation(
        &self,
        source: &AssetUri,
    ) -> Result<(Self, HashSet<AssetUuid>), AssetRegistryError> {
        let source = source_locator(source);
        let removed_uuids = self
            .source_entries(&source)
            .into_iter()
            .map(|entry| entry.uuid())
            .collect::<HashSet<_>>();
        if removed_uuids.is_empty() {
            return Err(AssetRegistryError::AssetPathNotFound { path: source });
        }
        let mut candidate = self.clone();
        candidate.remove_source_path(&source);
        Ok((candidate, removed_uuids))
    }
}

#[cfg(test)]
#[path = "tests/deletion.rs"]
mod tests;
