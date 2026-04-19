use std::path::PathBuf;

use zircon_runtime::asset::{AssetUri, AssetUuid};

use crate::{AssetCatalogRecord, PreviewArtifactKey};

use super::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub fn record_by_uuid(&self, asset_uuid: AssetUuid) -> Option<AssetCatalogRecord> {
        self.state
            .read()
            .expect("editor asset state lock poisoned")
            .catalog_by_uuid
            .get(&asset_uuid)
            .cloned()
    }

    pub fn record_by_locator(&self, locator: &AssetUri) -> Option<AssetCatalogRecord> {
        let state = self.state.read().expect("editor asset state lock poisoned");
        let asset_uuid = state.uuid_by_locator.get(locator)?;
        state.catalog_by_uuid.get(asset_uuid).cloned()
    }

    pub fn list_catalog(&self) -> Vec<AssetCatalogRecord> {
        let state = self.state.read().expect("editor asset state lock poisoned");
        let mut records = state.catalog_by_uuid.values().cloned().collect::<Vec<_>>();
        records.sort_by(|left, right| left.locator.to_string().cmp(&right.locator.to_string()));
        records
    }

    pub fn direct_references(&self, asset_uuid: AssetUuid) -> Vec<AssetCatalogRecord> {
        let state = self.state.read().expect("editor asset state lock poisoned");
        state
            .reference_graph
            .outgoing(asset_uuid)
            .into_iter()
            .filter_map(|target| state.catalog_by_uuid.get(&target).cloned())
            .collect()
    }

    pub fn referenced_by(&self, asset_uuid: AssetUuid) -> Vec<AssetCatalogRecord> {
        let state = self.state.read().expect("editor asset state lock poisoned");
        state
            .reference_graph
            .incoming(asset_uuid)
            .into_iter()
            .filter_map(|source| state.catalog_by_uuid.get(&source).cloned())
            .collect()
    }

    pub fn preview_artifact_path(&self, asset_uuid: AssetUuid) -> PathBuf {
        let state = self.state.read().expect("editor asset state lock poisoned");
        state
            .catalog_by_uuid
            .get(&asset_uuid)
            .map(|record| record.preview_artifact_path.clone())
            .or_else(|| {
                state
                    .preview_cache
                    .as_ref()
                    .map(|cache| cache.path_for(&PreviewArtifactKey::thumbnail(asset_uuid)))
            })
            .unwrap_or_default()
    }
}
