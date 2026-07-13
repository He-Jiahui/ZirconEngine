use crate::asset::{AssetId, AssetKind, AssetReference, AssetUri, AssetUuid};

use super::{AssetRegistryEntry, AssetRegistryError, AssetRegistryFilter, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// UE `GetAssetsByClass` equivalent.
    pub fn get_assets_by_type(&self, type_marker: AssetKind) -> Vec<&AssetRegistryEntry> {
        self.sorted_matches(|entry| entry.type_marker() == type_marker)
    }

    /// UE `GetAssets(FARFilter)` equivalent.
    pub fn get_assets(&self, filter: &AssetRegistryFilter) -> Vec<&AssetRegistryEntry> {
        self.sorted_matches(|entry| {
            filter
                .type_marker
                .is_none_or(|type_marker| entry.type_marker() == type_marker)
                && filter
                    .required_tags
                    .iter()
                    .all(|tag| entry.tags().contains(tag))
                && filter
                    .path_prefix
                    .as_deref()
                    .is_none_or(|prefix| entry.path().path().starts_with(prefix))
                && filter
                    .package_id
                    .as_deref()
                    .is_none_or(|package_id| entry.path().package_id() == Some(package_id))
        })
    }

    /// UE uuid dependency signature.
    pub fn get_dependencies_by_uuid(&self, uuid: AssetUuid) -> Vec<AssetUuid> {
        self.entry_by_uuid(uuid)
            .map(|entry| entry.dependencies().to_vec())
            .unwrap_or_default()
    }

    /// UE path/package dependency signature.
    pub fn get_dependencies_by_path(&self, path: &AssetUri) -> Vec<AssetUuid> {
        self.entry_by_path(path)
            .map(|entry| entry.dependencies().to_vec())
            .unwrap_or_default()
    }

    /// UE uuid reverse-reference signature.
    pub fn get_referencers_by_uuid(&self, uuid: AssetUuid) -> Vec<AssetUuid> {
        let mut referencers = self
            .entries_by_uuid
            .values()
            .filter(|entry| entry.dependencies().contains(&uuid))
            .map(AssetRegistryEntry::uuid)
            .collect::<Vec<_>>();
        referencers.sort_by_key(ToString::to_string);
        referencers
    }

    /// UE path/package reverse-reference signature.
    pub fn get_referencers_by_path(&self, path: &AssetUri) -> Vec<AssetUuid> {
        self.entry_by_path(path)
            .map(|entry| self.get_referencers_by_uuid(entry.uuid()))
            .unwrap_or_default()
    }

    pub fn resolve_asset_id_by_uuid(&self, uuid: AssetUuid) -> Result<AssetId, AssetRegistryError> {
        self.entries_by_uuid
            .contains_key(&uuid)
            .then(|| AssetId::from_asset_uuid(uuid))
            .ok_or(AssetRegistryError::AssetUuidNotFound { uuid })
    }

    pub fn resolve_asset_id_by_path(&self, path: &AssetUri) -> Result<AssetId, AssetRegistryError> {
        self.entry_by_path(path)
            .map(|entry| AssetId::from_asset_uuid(entry.uuid()))
            .ok_or_else(|| AssetRegistryError::AssetPathNotFound { path: path.clone() })
    }

    pub fn resolve_asset_id_for_reference(
        &self,
        uuid: AssetUuid,
        path: &AssetUri,
    ) -> Result<AssetId, AssetRegistryError> {
        self.resolve_asset_id_by_uuid(uuid)
            .or_else(|_| self.resolve_asset_id_by_path(path))
            .map_err(|_| AssetRegistryError::AssetReferenceNotFound {
                uuid,
                path: path.clone(),
            })
    }

    pub fn resolve_reference_by_asset_id(
        &self,
        id: AssetId,
    ) -> Result<AssetReference, AssetRegistryError> {
        self.entries_by_uuid
            .values()
            .find(|entry| AssetId::from_asset_uuid(entry.uuid()) == id)
            .map(|entry| AssetReference::new(entry.uuid(), entry.path().clone()))
            .ok_or(AssetRegistryError::AssetIdNotFound { id })
    }

    pub fn stale_path_for_uuid(&self, uuid: AssetUuid, path: &AssetUri) -> Option<&AssetUri> {
        self.entry_by_uuid(uuid)
            .map(AssetRegistryEntry::path)
            .filter(|current| *current != path)
    }

    fn sorted_matches(
        &self,
        predicate: impl Fn(&AssetRegistryEntry) -> bool,
    ) -> Vec<&AssetRegistryEntry> {
        let mut entries = self
            .entries_by_uuid
            .values()
            .filter(|entry| predicate(entry))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.path().cmp(right.path()));
        entries
    }
}
