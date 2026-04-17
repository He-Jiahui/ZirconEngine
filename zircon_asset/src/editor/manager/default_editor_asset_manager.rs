use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crossbeam_channel::unbounded;
use zircon_core::{ChannelReceiver, ChannelSender, CoreError};

use super::folder_projection::build_folder_records;
use super::preview_refresh::display_name_for_locator;
use super::super::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetChangeRecord,
    EditorAssetDetailsRecord, EditorAssetManager, EditorAssetReferenceRecord,
};
use crate::{
    AssetCatalogRecord, AssetImportError, AssetReference, AssetUri, AssetUuid, PreviewArtifactKey,
    PreviewCache, PreviewScheduler, ProjectManager, ReferenceGraph,
};

#[derive(Clone, Debug, Default)]
pub struct DefaultEditorAssetManager {
    pub(super) state: Arc<RwLock<EditorAssetState>>,
    change_subscribers: Arc<Mutex<Vec<ChannelSender<EditorAssetChangeRecord>>>>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct EditorAssetState {
    pub(super) project_root: Option<PathBuf>,
    pub(super) assets_root: Option<PathBuf>,
    pub(super) library_root: Option<PathBuf>,
    pub(super) project_name: String,
    pub(super) default_scene_uri: Option<AssetUri>,
    pub(super) catalog_revision: u64,
    pub(super) project: Option<ProjectManager>,
    pub(super) catalog_by_uuid: HashMap<AssetUuid, AssetCatalogRecord>,
    pub(super) uuid_by_locator: HashMap<AssetUri, AssetUuid>,
    pub(super) reference_graph: ReferenceGraph,
    pub(super) preview_cache: Option<PreviewCache>,
    pub(super) preview_scheduler: PreviewScheduler,
}

impl DefaultEditorAssetManager {
    pub fn open_project(&self, root: impl AsRef<Path>) -> Result<(), AssetImportError> {
        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        self.sync_from_project(project)
    }

    pub fn record_by_uuid(&self, asset_uuid: AssetUuid) -> Option<AssetCatalogRecord> {
        self.state
            .read()
            .expect("editor asset state lock poisoned")
            .catalog_by_uuid
            .get(&asset_uuid)
            .cloned()
    }

    pub fn record_by_locator(&self, locator: &crate::AssetUri) -> Option<AssetCatalogRecord> {
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

    pub(super) fn broadcast(&self, change: EditorAssetChangeRecord) {
        let mut subscribers = self
            .change_subscribers
            .lock()
            .expect("editor asset subscriber lock poisoned");
        subscribers.retain(|sender| sender.send(change.clone()).is_ok());
    }
}

impl EditorAssetManager for DefaultEditorAssetManager {
    fn catalog_snapshot(&self) -> EditorAssetCatalogSnapshotRecord {
        let state = self.state.read().expect("editor asset state lock poisoned");
        let mut assets = state
            .catalog_by_uuid
            .values()
            .map(record_to_facade)
            .collect::<Vec<_>>();
        assets.sort_by(|left, right| left.locator.cmp(&right.locator));

        EditorAssetCatalogSnapshotRecord {
            project_name: state.project_name.clone(),
            project_root: state
                .project_root
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            assets_root: state
                .assets_root
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            library_root: state
                .library_root
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            default_scene_uri: state
                .default_scene_uri
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            catalog_revision: state.catalog_revision,
            folders: build_folder_records(&state),
            assets,
        }
    }

    fn asset_details(&self, uuid: &str) -> Option<EditorAssetDetailsRecord> {
        let asset_uuid = uuid.parse::<AssetUuid>().ok()?;
        let state = self.state.read().expect("editor asset state lock poisoned");
        let record = state.catalog_by_uuid.get(&asset_uuid)?;

        let mut direct_references = record
            .direct_references
            .iter()
            .map(|reference| reference_to_facade(reference, &state))
            .collect::<Vec<_>>();
        direct_references.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then(left.locator.cmp(&right.locator))
        });

        let mut referenced_by = state
            .reference_graph
            .incoming(asset_uuid)
            .into_iter()
            .filter_map(|source_uuid| state.catalog_by_uuid.get(&source_uuid))
            .map(|source| EditorAssetReferenceRecord {
                uuid: source.asset_uuid.to_string(),
                locator: source.locator.to_string(),
                display_name: source.display_name.clone(),
                kind: Some(source.kind),
                known_project_asset: true,
            })
            .collect::<Vec<_>>();
        referenced_by.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then(left.locator.cmp(&right.locator))
        });

        Some(EditorAssetDetailsRecord {
            asset: record_to_facade(record),
            direct_references,
            referenced_by,
            editor_adapter: record.meta.editor_adapter.clone(),
        })
    }

    fn subscribe_editor_asset_changes(&self) -> ChannelReceiver<EditorAssetChangeRecord> {
        let (sender, receiver) = unbounded();
        self.change_subscribers
            .lock()
            .expect("editor asset subscriber lock poisoned")
            .push(sender);
        receiver
    }

    fn mark_preview_dirty(
        &self,
        uuid: &str,
    ) -> Result<Option<EditorAssetDetailsRecord>, CoreError> {
        let asset_uuid = parse_uuid(uuid)?;
        self.mark_preview_dirty(asset_uuid)
            .map_err(editor_asset_error)?;
        Ok(self.asset_details(uuid))
    }

    fn request_preview_refresh(
        &self,
        uuid: &str,
        visible: bool,
    ) -> Result<Option<EditorAssetDetailsRecord>, CoreError> {
        let asset_uuid = parse_uuid(uuid)?;
        self.request_preview_refresh(asset_uuid, visible)
            .map_err(editor_asset_error)?;
        Ok(self.asset_details(uuid))
    }
}

fn parse_uuid(uuid: &str) -> Result<AssetUuid, CoreError> {
    uuid.parse::<AssetUuid>().map_err(|error| {
        CoreError::Initialization(
            crate::EDITOR_ASSET_MANAGER_NAME.to_string(),
            format!("invalid asset uuid {uuid}: {error}"),
        )
    })
}

fn editor_asset_error(error: AssetImportError) -> CoreError {
    CoreError::Initialization(
        crate::EDITOR_ASSET_MANAGER_NAME.to_string(),
        error.to_string(),
    )
}

fn record_to_facade(record: &AssetCatalogRecord) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: record.asset_uuid.to_string(),
        id: record.asset_id.to_string(),
        locator: record.locator.to_string(),
        kind: record.kind,
        display_name: record.display_name.clone(),
        file_name: record.file_name.clone(),
        extension: record.extension.clone(),
        preview_state: record.preview_state,
        meta_path: record.meta_path.to_string_lossy().into_owned(),
        preview_artifact_path: record.preview_artifact_path.to_string_lossy().into_owned(),
        source_mtime_unix_ms: record.source_mtime_unix_ms,
        source_hash: record.source_hash.clone(),
        dirty: record.dirty,
        diagnostics: record.diagnostics.clone(),
        direct_reference_uuids: record
            .direct_references
            .iter()
            .map(|reference| reference.uuid.to_string())
            .collect(),
    }
}

fn reference_to_facade(
    reference: &AssetReference,
    state: &EditorAssetState,
) -> EditorAssetReferenceRecord {
    if let Some(record) = state.catalog_by_uuid.get(&reference.uuid).or_else(|| {
        state
            .uuid_by_locator
            .get(&reference.locator)
            .and_then(|uuid| state.catalog_by_uuid.get(uuid))
    }) {
        return EditorAssetReferenceRecord {
            uuid: record.asset_uuid.to_string(),
            locator: record.locator.to_string(),
            display_name: record.display_name.clone(),
            kind: Some(record.kind),
            known_project_asset: true,
        };
    }

    EditorAssetReferenceRecord {
        uuid: reference.uuid.to_string(),
        locator: reference.locator.to_string(),
        display_name: display_name_for_locator(&reference.locator),
        kind: None,
        known_project_asset: false,
    }
}
