use std::collections::HashSet;

use crate::asset::{AssetUri, AssetUuid};

use super::asset_registry_index::source_locator;
use super::{AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// Builds a source-deletion candidate after topology admission has succeeded.
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
mod tests {
    use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
    use crate::asset::{AssetKind, AssetUri, AssetUuid};

    #[test]
    fn source_deletion_removes_root_and_subassets_without_touching_other_sources() {
        let source = AssetUri::parse("res://models/ship.glb").unwrap();
        let root_uuid = AssetUuid::from_stable_label("delete-root");
        let mesh_uuid = AssetUuid::from_stable_label("delete-mesh");
        let other_uuid = AssetUuid::from_stable_label("delete-other");
        let index = AssetRegistryIndex::from_entries([
            AssetRegistryEntry::new(root_uuid, source.clone(), AssetKind::Model, "root"),
            AssetRegistryEntry::new(
                mesh_uuid,
                AssetUri::parse("res://models/ship.glb#mesh").unwrap(),
                AssetKind::Model,
                "mesh",
            ),
            AssetRegistryEntry::new(
                other_uuid,
                AssetUri::parse("res://models/other.glb").unwrap(),
                AssetKind::Model,
                "other",
            ),
        ])
        .unwrap();

        let (candidate, removed) = index.prepare_source_deletion_generation(&source).unwrap();

        assert_eq!(removed, [root_uuid, mesh_uuid].into_iter().collect());
        assert!(candidate.entry_by_uuid(root_uuid).is_none());
        assert!(candidate.entry_by_uuid(mesh_uuid).is_none());
        assert!(candidate.entry_by_uuid(other_uuid).is_some());
    }
}
