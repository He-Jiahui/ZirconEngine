use crate::asset::{AssetId, AssetReference, AssetUri, AssetUuid};

use super::ProjectManager;

impl ProjectManager {
    pub fn asset_id_for_uri(&self, uri: &AssetUri) -> Option<AssetId> {
        self.registry
            .get_by_locator(uri)
            .map(|metadata| metadata.id())
    }

    pub fn asset_id_for_uuid(&self, uuid: AssetUuid) -> Option<AssetId> {
        self.asset_ids_by_uuid.get(&uuid).copied()
    }

    pub fn asset_id_for_reference(&self, uuid: AssetUuid, locator: &AssetUri) -> Option<AssetId> {
        self.asset_id_for_uri(locator).or_else(|| {
            self.asset_ids_by_uuid
                .contains_key(&uuid)
                .then(|| AssetId::from_asset_uuid_label(uuid, locator.label()))
        })
    }

    pub fn asset_uri_for_id(&self, id: AssetId) -> Option<&AssetUri> {
        self.registry
            .get(id)
            .map(|metadata| metadata.primary_locator())
    }

    pub fn asset_reference_for_id(&self, id: AssetId) -> Option<AssetReference> {
        let locator = self.asset_uri_for_id(id)?.clone();
        let uuid = self
            .asset_uuids_by_id
            .get(&id)
            .copied()
            .unwrap_or_else(|| AssetUuid::from_stable_label(&locator.to_string()));
        Some(AssetReference::new(uuid, locator))
    }
}
