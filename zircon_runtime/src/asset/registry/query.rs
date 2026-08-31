use crate::asset::{AssetId, AssetKind, AssetReference, AssetUri, AssetUuid};

use super::{AssetRegistryEntry, AssetRegistryError, AssetRegistryFilter, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// UE `GetAssetsByClass` equivalent.
    pub fn get_assets_by_type(&self, type_marker: AssetKind) -> Vec<&AssetRegistryEntry> {
        self.sorted_type_matches(type_marker, |_| true)
    }

    /// UE `GetAssets(FARFilter)` equivalent.
    pub fn get_assets(&self, filter: &AssetRegistryFilter) -> Vec<&AssetRegistryEntry> {
        if let Some(type_marker) = filter.type_marker {
            return self.sorted_type_matches(type_marker, |entry| {
                entry_matches_filter(entry, filter)
            });
        }
        self.sorted_matches(|entry| entry_matches_filter(entry, filter))
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
            .referencers_by_uuid
            .get(&uuid)
            .into_iter()
            .flatten()
            .copied()
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
        // A path is a locator hint only. Falling back to it here can silently
        // bind a stale reference to an unrelated asset that reuses the path.
        self.resolve_asset_id_by_uuid(uuid).map_err(|_| {
            AssetRegistryError::AssetReferenceNotFound {
                uuid,
                path: path.clone(),
            }
        })
    }

    pub fn resolve_reference_by_asset_id(
        &self,
        id: AssetId,
    ) -> Result<AssetReference, AssetRegistryError> {
        self.uuid_by_asset_id
            .get(&id)
            .and_then(|uuid| self.entries_by_uuid.get(uuid))
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

    fn sorted_type_matches(
        &self,
        type_marker: AssetKind,
        predicate: impl Fn(&AssetRegistryEntry) -> bool,
    ) -> Vec<&AssetRegistryEntry> {
        let mut entries = self
            .uuids_by_type
            .get(&type_marker)
            .into_iter()
            .flatten()
            .filter_map(|uuid| self.entries_by_uuid.get(uuid))
            .filter(|entry| predicate(entry))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.path().cmp(right.path()));
        entries
    }
}

fn entry_matches_filter(entry: &AssetRegistryEntry, filter: &AssetRegistryFilter) -> bool {
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
}
