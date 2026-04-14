mod metadata;

use std::collections::HashMap;

pub use metadata::{AssetId, AssetKind, AssetMetadata};

use crate::AssetUri;

#[derive(Clone, Debug, Default)]
pub struct AssetRegistry {
    by_id: HashMap<AssetId, AssetMetadata>,
    id_by_uri: HashMap<AssetUri, AssetId>,
}

impl AssetRegistry {
    pub fn upsert(&mut self, metadata: AssetMetadata) -> Option<AssetMetadata> {
        if let Some(existing) = self.by_id.get(&metadata.id).cloned() {
            self.id_by_uri.remove(&existing.uri);
        }
        self.id_by_uri.insert(metadata.uri.clone(), metadata.id);
        self.by_id.insert(metadata.id, metadata)
    }

    pub fn get_by_id(&self, id: AssetId) -> Option<&AssetMetadata> {
        self.by_id.get(&id)
    }

    pub fn get_by_uri(&self, uri: &AssetUri) -> Option<&AssetMetadata> {
        self.id_by_uri.get(uri).and_then(|id| self.by_id.get(id))
    }

    pub fn values(&self) -> impl Iterator<Item = &AssetMetadata> {
        self.by_id.values()
    }

    pub fn rename(&mut self, from: &AssetUri, to: AssetUri) -> Result<AssetMetadata, String> {
        let Some(asset_id) = self.id_by_uri.remove(from) else {
            return Err(format!("missing asset metadata for {from}"));
        };
        let Some(metadata) = self.by_id.get_mut(&asset_id) else {
            return Err(format!("missing asset metadata for id {}", asset_id));
        };
        metadata.uri = to.clone();
        self.id_by_uri.insert(to, asset_id);
        Ok(metadata.clone())
    }

    pub fn remove(&mut self, uri: &AssetUri) -> Option<AssetMetadata> {
        let asset_id = self.id_by_uri.remove(uri)?;
        self.by_id.remove(&asset_id)
    }
}
