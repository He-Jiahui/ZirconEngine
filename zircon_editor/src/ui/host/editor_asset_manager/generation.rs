use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime::asset::project::AssetSourceUnit;
use zircon_runtime::asset::project::ProjectPaths;

use super::{
    AssetCatalogRecord, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetDetailsRecord, EditorAssetFolderRecord, EditorAssetReferenceRecord,
    EditorAssetSubassetRecord,
};

#[derive(Clone, Debug)]
pub struct EditorAssetDetailsGeneration {
    pub asset: Arc<EditorAssetCatalogRecord>,
    pub direct_references: Arc<[EditorAssetReferenceRecord]>,
    pub referenced_by: Arc<[EditorAssetReferenceRecord]>,
    pub package_id: Option<Arc<str>>,
    pub unit: AssetSourceUnit,
    pub included_files: Arc<[String]>,
    pub subassets: Arc<[EditorAssetSubassetRecord]>,
}

impl EditorAssetDetailsGeneration {
    pub(crate) fn with_asset(&self, asset: Arc<EditorAssetCatalogRecord>) -> Self {
        Self {
            asset,
            direct_references: Arc::clone(&self.direct_references),
            referenced_by: Arc::clone(&self.referenced_by),
            package_id: self.package_id.clone(),
            unit: self.unit,
            included_files: Arc::clone(&self.included_files),
            subassets: Arc::clone(&self.subassets),
        }
    }
}

impl From<EditorAssetDetailsRecord> for EditorAssetDetailsGeneration {
    fn from(record: EditorAssetDetailsRecord) -> Self {
        Self {
            asset: Arc::new(record.asset),
            direct_references: record.direct_references.into(),
            referenced_by: record.referenced_by.into(),
            package_id: record.package_id.map(Arc::from),
            unit: record.unit,
            included_files: record.included_files.into(),
            subassets: record.subassets.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EditorAssetCatalogGeneration {
    pub project_name: Arc<str>,
    pub project_root: Arc<str>,
    pub assets_root: Arc<str>,
    pub cache_root: Arc<str>,
    pub default_scene_uri: Arc<str>,
    pub catalog_revision: u64,
    pub publish_epoch: u64,
    pub folders: Arc<[EditorAssetFolderRecord]>,
    pub assets: Arc<[Arc<EditorAssetCatalogRecord>]>,
    asset_index_by_uuid: Arc<HashMap<String, usize>>,
    asset_index_by_locator: Arc<HashMap<String, usize>>,
    folder_index_by_id: Arc<HashMap<String, usize>>,
    details_by_asset_index: Arc<[Option<Arc<EditorAssetDetailsGeneration>>]>,
    // Runtime-derived editor rows are kept with their public projection so preview completion
    // can COW one row without reviving mutable catalog maps in manager state.
    catalog_records: Arc<[Option<Arc<AssetCatalogRecord>>]>,
}

impl Default for EditorAssetCatalogGeneration {
    fn default() -> Self {
        Self::from_snapshot_record(EditorAssetCatalogSnapshotRecord::default(), 0)
    }
}

impl EditorAssetCatalogGeneration {
    pub(crate) fn next_catalog_identity(&self) -> (u64, u64) {
        let catalog_revision = self
            .catalog_revision
            .checked_add(1)
            .expect("editor asset catalog revision exhausted");
        (catalog_revision, self.next_publish_epoch())
    }

    pub(crate) fn next_publish_epoch(&self) -> u64 {
        self.publish_epoch
            .checked_add(1)
            .expect("editor asset catalog publish epoch exhausted")
    }

    pub(crate) fn from_snapshot_record(
        snapshot: EditorAssetCatalogSnapshotRecord,
        publish_epoch: u64,
    ) -> Self {
        let assets = snapshot
            .assets
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let details = vec![None; assets.len()];
        let catalog_record_count = assets.len();
        Self::from_parts(
            snapshot.project_name,
            snapshot.project_root,
            snapshot.assets_root,
            snapshot.cache_root,
            snapshot.default_scene_uri,
            snapshot.catalog_revision,
            publish_epoch,
            snapshot.folders,
            assets,
            details,
            vec![None; catalog_record_count],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        project_name: String,
        project_root: String,
        assets_root: String,
        cache_root: String,
        default_scene_uri: String,
        catalog_revision: u64,
        publish_epoch: u64,
        folders: Vec<EditorAssetFolderRecord>,
        assets: Vec<Arc<EditorAssetCatalogRecord>>,
        details_by_asset_index: Vec<Option<Arc<EditorAssetDetailsGeneration>>>,
        catalog_records: Vec<Option<Arc<AssetCatalogRecord>>>,
    ) -> Self {
        debug_assert_eq!(assets.len(), details_by_asset_index.len());
        debug_assert_eq!(assets.len(), catalog_records.len());
        let asset_index_by_uuid = assets
            .iter()
            .enumerate()
            .map(|(index, asset)| (asset.uuid.clone(), index))
            .collect();
        let asset_index_by_locator = assets
            .iter()
            .enumerate()
            .map(|(index, asset)| (asset.locator.clone(), index))
            .collect();
        let folder_index_by_id = folders
            .iter()
            .enumerate()
            .map(|(index, folder)| (folder.folder_id.clone(), index))
            .collect();
        Self {
            project_name: Arc::from(project_name),
            project_root: Arc::from(project_root),
            assets_root: Arc::from(assets_root),
            cache_root: Arc::from(cache_root),
            default_scene_uri: Arc::from(default_scene_uri),
            catalog_revision,
            publish_epoch,
            folders: folders.into(),
            assets: assets.into(),
            asset_index_by_uuid: Arc::new(asset_index_by_uuid),
            asset_index_by_locator: Arc::new(asset_index_by_locator),
            folder_index_by_id: Arc::new(folder_index_by_id),
            details_by_asset_index: details_by_asset_index.into(),
            catalog_records: catalog_records.into(),
        }
    }

    pub fn asset(&self, uuid: &str) -> Option<&EditorAssetCatalogRecord> {
        let index = *self.asset_index_by_uuid.get(uuid)?;
        self.assets.get(index).map(Arc::as_ref)
    }

    pub(crate) fn asset_shared(&self, uuid: &str) -> Option<Arc<EditorAssetCatalogRecord>> {
        let index = *self.asset_index_by_uuid.get(uuid)?;
        self.assets.get(index).map(Arc::clone)
    }

    pub(crate) fn catalog_record(&self, uuid: &str) -> Option<Arc<AssetCatalogRecord>> {
        let index = *self.asset_index_by_uuid.get(uuid)?;
        self.catalog_records.get(index)?.as_ref().map(Arc::clone)
    }

    pub fn asset_by_locator(&self, locator: &str) -> Option<&EditorAssetCatalogRecord> {
        let index = *self.asset_index_by_locator.get(locator)?;
        self.assets.get(index).map(Arc::as_ref)
    }

    pub fn folder(&self, folder_id: &str) -> Option<&EditorAssetFolderRecord> {
        let index = *self.folder_index_by_id.get(folder_id)?;
        self.folders.get(index)
    }

    pub fn details(&self, uuid: &str) -> Option<Arc<EditorAssetDetailsGeneration>> {
        let index = *self.asset_index_by_uuid.get(uuid)?;
        self.details_by_asset_index
            .get(index)?
            .as_ref()
            .map(Arc::clone)
    }

    pub(crate) fn updated_asset(
        &self,
        updated: Arc<EditorAssetCatalogRecord>,
        publish_epoch: u64,
    ) -> Option<Self> {
        let index = *self.asset_index_by_uuid.get(updated.uuid.as_str())?;
        let mut assets = self.assets.iter().cloned().collect::<Vec<_>>();
        assets[index] = Arc::clone(&updated);
        let mut details = self
            .details_by_asset_index
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(current_details) = details[index].as_ref() {
            details[index] = Some(Arc::new(current_details.with_asset(updated)));
        }
        Some(Self {
            project_name: Arc::clone(&self.project_name),
            project_root: Arc::clone(&self.project_root),
            assets_root: Arc::clone(&self.assets_root),
            cache_root: Arc::clone(&self.cache_root),
            default_scene_uri: Arc::clone(&self.default_scene_uri),
            catalog_revision: self.catalog_revision,
            publish_epoch,
            folders: Arc::clone(&self.folders),
            assets: assets.into(),
            asset_index_by_uuid: Arc::clone(&self.asset_index_by_uuid),
            asset_index_by_locator: Arc::clone(&self.asset_index_by_locator),
            folder_index_by_id: Arc::clone(&self.folder_index_by_id),
            details_by_asset_index: details.into(),
            catalog_records: Arc::clone(&self.catalog_records),
        })
    }

    pub(crate) fn updated_catalog_record(
        &self,
        updated: AssetCatalogRecord,
        publish_epoch: u64,
    ) -> Option<Self> {
        let index = *self
            .asset_index_by_uuid
            .get(&updated.asset_uuid.to_string())?;
        let current_catalog = self.catalog_records.get(index)?.as_ref()?;
        let current_asset = self.assets.get(index)?;
        let updated_asset = Arc::new(public_record_with_current_references(
            current_asset,
            &updated,
        ));
        let mut assets = self.assets.iter().cloned().collect::<Vec<_>>();
        assets[index] = Arc::clone(&updated_asset);
        let mut details = self
            .details_by_asset_index
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(current_details) = details[index].as_ref() {
            details[index] = Some(Arc::new(current_details.with_asset(updated_asset)));
        }
        let mut catalog_records = self.catalog_records.iter().cloned().collect::<Vec<_>>();
        catalog_records[index] = Some(Arc::new(updated));
        debug_assert_eq!(current_catalog.asset_uuid.to_string(), assets[index].uuid);

        Some(Self {
            project_name: Arc::clone(&self.project_name),
            project_root: Arc::clone(&self.project_root),
            assets_root: Arc::clone(&self.assets_root),
            cache_root: Arc::clone(&self.cache_root),
            default_scene_uri: Arc::clone(&self.default_scene_uri),
            catalog_revision: self.catalog_revision,
            publish_epoch,
            folders: Arc::clone(&self.folders),
            assets: assets.into(),
            asset_index_by_uuid: Arc::clone(&self.asset_index_by_uuid),
            asset_index_by_locator: Arc::clone(&self.asset_index_by_locator),
            folder_index_by_id: Arc::clone(&self.folder_index_by_id),
            details_by_asset_index: details.into(),
            catalog_records: catalog_records.into(),
        })
    }

    /// Applies several Runtime-authoritative catalog record changes in one COW publication.
    ///
    /// Watch batches can contain many paths. Repeating `updated_catalog_record` for each path
    /// would copy the generation's pointer arrays once per path, so resolve all UUIDs before one
    /// replacement allocation instead.
    pub(crate) fn updated_catalog_records(
        &self,
        updates: impl IntoIterator<Item = AssetCatalogRecord>,
        publish_epoch: u64,
    ) -> Option<Self> {
        let mut updates_by_index = HashMap::new();
        for updated in updates {
            let Some(index) = self
                .asset_index_by_uuid
                .get(&updated.asset_uuid.to_string())
                .copied()
            else {
                continue;
            };
            let Some(current_catalog) = self.catalog_records.get(index).and_then(Option::as_ref)
            else {
                continue;
            };
            if current_catalog.as_ref() != &updated {
                updates_by_index.insert(index, updated);
            }
        }
        if updates_by_index.is_empty() {
            return None;
        }

        let mut updates = updates_by_index.into_iter().collect::<Vec<_>>();
        updates.sort_by_key(|(index, _)| *index);
        let mut assets = self.assets.iter().cloned().collect::<Vec<_>>();
        let mut details = self
            .details_by_asset_index
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut catalog_records = self.catalog_records.iter().cloned().collect::<Vec<_>>();
        for (index, updated) in updates {
            let current_asset = self
                .assets
                .get(index)
                .expect("catalog UUID index must address an asset row");
            let updated_asset = Arc::new(public_record_with_current_references(
                current_asset,
                &updated,
            ));
            assets[index] = Arc::clone(&updated_asset);
            if let Some(current_details) = details[index].as_ref() {
                details[index] = Some(Arc::new(current_details.with_asset(updated_asset)));
            }
            catalog_records[index] = Some(Arc::new(updated));
        }

        Some(Self {
            project_name: Arc::clone(&self.project_name),
            project_root: Arc::clone(&self.project_root),
            assets_root: Arc::clone(&self.assets_root),
            cache_root: Arc::clone(&self.cache_root),
            default_scene_uri: Arc::clone(&self.default_scene_uri),
            catalog_revision: self.catalog_revision,
            publish_epoch,
            folders: Arc::clone(&self.folders),
            assets: assets.into(),
            asset_index_by_uuid: Arc::clone(&self.asset_index_by_uuid),
            asset_index_by_locator: Arc::clone(&self.asset_index_by_locator),
            folder_index_by_id: Arc::clone(&self.folder_index_by_id),
            details_by_asset_index: details.into(),
            catalog_records: catalog_records.into(),
        })
    }
}

fn public_record_with_current_references(
    current: &EditorAssetCatalogRecord,
    updated: &AssetCatalogRecord,
) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: updated.asset_uuid.to_string(),
        id: updated.asset_id.to_string(),
        locator: updated.locator.to_string(),
        kind: updated.kind,
        display_name: updated.display_name.clone(),
        file_name: updated.file_name.clone(),
        extension: updated.extension.clone(),
        preview_state: updated.preview_state,
        meta_path: ProjectPaths::display_path(&updated.meta_path)
            .to_string_lossy()
            .into_owned(),
        preview_artifact_path: ProjectPaths::display_path(&updated.preview_artifact_path)
            .to_string_lossy()
            .into_owned(),
        source_mtime_unix_ms: updated.source_mtime_unix_ms,
        source_hash: updated.source_hash.clone(),
        dirty: updated.dirty,
        diagnostics: updated.diagnostics.clone(),
        // A preview job is admitted only while source hash and metadata location match, so
        // references cannot change during this one-row publication.
        direct_reference_uuids: current.direct_reference_uuids.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::EditorAssetCatalogGeneration;

    #[test]
    fn catalog_identity_advances_revision_and_publish_epoch_together() {
        let generation = EditorAssetCatalogGeneration::default();

        assert_eq!(generation.next_catalog_identity(), (1, 1));
    }

    #[test]
    #[should_panic(expected = "editor asset catalog revision exhausted")]
    fn catalog_revision_exhaustion_never_reuses_an_identity() {
        let mut generation = EditorAssetCatalogGeneration::default();
        generation.catalog_revision = u64::MAX;

        let _ = generation.next_catalog_identity();
    }

    #[test]
    #[should_panic(expected = "editor asset catalog publish epoch exhausted")]
    fn publish_epoch_exhaustion_never_reuses_an_identity() {
        let mut generation = EditorAssetCatalogGeneration::default();
        generation.publish_epoch = u64::MAX;

        let _ = generation.next_publish_epoch();
    }
}
