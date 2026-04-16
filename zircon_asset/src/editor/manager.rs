use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crossbeam_channel::unbounded;
use zircon_core::{ChannelReceiver, ChannelSender, CoreError};
use zircon_manager::{
    AssetRecordKind, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetChangeKind, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetReferenceRecord, PreviewStateRecord,
};

use super::preview::PreviewPalette;
use crate::{
    AssetCatalogRecord, AssetImportError, AssetMetaDocument, AssetReference, AssetUri, AssetUuid,
    ImportedAsset, PreviewArtifactKey, PreviewCache, PreviewScheduler, PreviewState,
    ProjectManager, ReferenceGraph, ResourceId,
};

#[derive(Clone, Debug, Default)]
pub struct DefaultEditorAssetManager {
    state: Arc<RwLock<EditorAssetState>>,
    change_subscribers: Arc<Mutex<Vec<ChannelSender<EditorAssetChangeRecord>>>>,
}

#[derive(Clone, Debug, Default)]
struct EditorAssetState {
    project_root: Option<PathBuf>,
    assets_root: Option<PathBuf>,
    library_root: Option<PathBuf>,
    project_name: String,
    default_scene_uri: Option<AssetUri>,
    catalog_revision: u64,
    project: Option<ProjectManager>,
    catalog_by_uuid: HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: HashMap<AssetUri, AssetUuid>,
    reference_graph: ReferenceGraph,
    preview_cache: Option<PreviewCache>,
    preview_scheduler: PreviewScheduler,
}

#[derive(Clone, Debug, Default)]
struct FolderBuilder {
    parent_folder_id: Option<String>,
    locator_prefix: String,
    display_name: String,
    child_folder_ids: Vec<String>,
    direct_asset_uuids: Vec<String>,
    recursive_asset_count: usize,
}

impl DefaultEditorAssetManager {
    pub fn open_project(&self, root: impl AsRef<Path>) -> Result<(), AssetImportError> {
        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        self.sync_from_project(project)
    }

    pub fn sync_from_project(&self, project: ProjectManager) -> Result<(), AssetImportError> {
        let preview_cache = PreviewCache::new(project.paths().library_root())?;
        let mut catalog_by_uuid = HashMap::new();
        let mut uuid_by_locator = HashMap::new();
        let mut preview_scheduler = PreviewScheduler::default();

        for metadata in project.registry().values() {
            let locator = metadata.primary_locator().clone();
            let source_path = project.source_path_for_uri(&locator)?;
            let meta_path = meta_path_for_source(&source_path);
            let meta = AssetMetaDocument::load(&meta_path)?;
            let preview_state = meta.preview_state;
            let imported = project.load_artifact_by_id(metadata.id())?;
            let direct_references = direct_references(&imported);
            let preview_artifact_path =
                preview_cache.path_for(&PreviewArtifactKey::thumbnail(meta.asset_uuid));
            let file_name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            let extension = source_path
                .extension()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let diagnostics = metadata
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>();

            let record = AssetCatalogRecord {
                asset_uuid: meta.asset_uuid,
                asset_id: metadata.id(),
                locator: locator.clone(),
                kind: metadata.kind,
                display_name: display_name_for_path(&source_path, &locator),
                file_name,
                extension,
                meta_path,
                meta,
                source_mtime_unix_ms: preview_source_mtime(&source_path),
                source_hash: metadata.source_hash.clone(),
                preview_state,
                preview_artifact_path,
                dirty: preview_state == PreviewState::Dirty,
                diagnostics,
                direct_references,
            };
            if record.dirty {
                preview_scheduler.mark_dirty(record.asset_uuid);
            }

            uuid_by_locator.insert(locator, record.asset_uuid);
            catalog_by_uuid.insert(record.asset_uuid, record);
        }

        let reference_graph = ReferenceGraph::rebuild(catalog_by_uuid.values());
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            state.project_root = Some(project.paths().root().to_path_buf());
            state.assets_root = Some(project.paths().assets_root().to_path_buf());
            state.library_root = Some(project.paths().library_root().to_path_buf());
            state.project_name = project.manifest().name.clone();
            state.default_scene_uri = Some(project.manifest().default_scene.clone());
            state.catalog_revision += 1;
            state.project = Some(project);
            state.catalog_by_uuid = catalog_by_uuid;
            state.uuid_by_locator = uuid_by_locator;
            state.reference_graph = reference_graph;
            state.preview_cache = Some(preview_cache);
            state.preview_scheduler = preview_scheduler;

            EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::CatalogChanged,
                catalog_revision: state.catalog_revision,
                uuid: None,
                locator: None,
            }
        };
        self.broadcast(change);
        Ok(())
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

    pub fn mark_preview_dirty(
        &self,
        asset_uuid: AssetUuid,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let updated = {
                let Some(record) = state.catalog_by_uuid.get_mut(&asset_uuid) else {
                    return Ok(None);
                };
                record.preview_state = PreviewState::Dirty;
                record.dirty = true;
                record.meta.preview_state = PreviewState::Dirty;
                record.meta.save(&record.meta_path)?;
                record.clone()
            };
            state.preview_scheduler.mark_dirty(asset_uuid);
            Some(EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: state.catalog_revision,
                uuid: Some(updated.asset_uuid.to_string()),
                locator: Some(updated.locator.to_string()),
            })
        };
        if let Some(change) = change {
            self.broadcast(change);
        }
        Ok(self.record_by_uuid(asset_uuid))
    }

    pub fn request_preview_refresh(
        &self,
        asset_uuid: AssetUuid,
        visible: bool,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let should_refresh = state.preview_scheduler.request_refresh(asset_uuid, visible);
            let catalog_revision = state.catalog_revision;
            let cache = state.preview_cache.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("preview cache is not initialized".to_string())
            })?;
            let project = state.project.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("editor project is not initialized".to_string())
            })?;
            let Some(record) = state.catalog_by_uuid.get_mut(&asset_uuid) else {
                return Ok(None);
            };
            if !should_refresh {
                return Ok(Some(record.clone()));
            }

            match generate_preview_artifact(&project, record, &cache) {
                Ok(path) => {
                    record.preview_artifact_path = path;
                    record.preview_state = PreviewState::Ready;
                    record.dirty = false;
                    record.meta.preview_state = PreviewState::Ready;
                    record.meta.save(&record.meta_path)?;
                }
                Err(error) => {
                    record.preview_state = PreviewState::Error;
                    record.dirty = false;
                    record.meta.preview_state = PreviewState::Error;
                    record.meta.save(&record.meta_path)?;
                    return Err(error);
                }
            }

            Some(EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision,
                uuid: Some(record.asset_uuid.to_string()),
                locator: Some(record.locator.to_string()),
            })
        };
        if let Some(change) = change {
            self.broadcast(change);
        }
        Ok(self.record_by_uuid(asset_uuid))
    }

    fn broadcast(&self, change: EditorAssetChangeRecord) {
        let mut subscribers = self
            .change_subscribers
            .lock()
            .expect("editor asset subscriber lock poisoned");
        subscribers.retain(|sender| sender.send(change.clone()).is_ok());
    }
}

impl zircon_manager::EditorAssetManager for DefaultEditorAssetManager {
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
                kind: Some(asset_record_kind(source.kind)),
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
            zircon_manager::EDITOR_ASSET_MANAGER_NAME.to_string(),
            format!("invalid asset uuid {uuid}: {error}"),
        )
    })
}

fn editor_asset_error(error: AssetImportError) -> CoreError {
    CoreError::Initialization(
        zircon_manager::EDITOR_ASSET_MANAGER_NAME.to_string(),
        error.to_string(),
    )
}

fn record_to_facade(record: &AssetCatalogRecord) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: record.asset_uuid.to_string(),
        id: record.asset_id.to_string(),
        locator: record.locator.to_string(),
        kind: asset_record_kind(record.kind),
        display_name: record.display_name.clone(),
        file_name: record.file_name.clone(),
        extension: record.extension.clone(),
        preview_state: preview_state_record(record.preview_state),
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
            kind: Some(asset_record_kind(record.kind)),
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

fn build_folder_records(state: &EditorAssetState) -> Vec<EditorAssetFolderRecord> {
    let mut folders = BTreeMap::<String, FolderBuilder>::new();
    folders.insert(
        "res://".to_string(),
        FolderBuilder {
            parent_folder_id: None,
            locator_prefix: "res://".to_string(),
            display_name: "Assets".to_string(),
            ..FolderBuilder::default()
        },
    );

    for record in state
        .catalog_by_uuid
        .values()
        .filter(|record| record.locator.scheme() == crate::AssetUriScheme::Res)
    {
        let path_segments = record.locator.path().split('/').collect::<Vec<_>>();
        let folder_segments = if path_segments.len() > 1 {
            &path_segments[..path_segments.len() - 1]
        } else {
            &[][..]
        };
        let mut parent_id = "res://".to_string();
        for segment in folder_segments {
            let folder_id = if parent_id == "res://" {
                format!("res://{segment}")
            } else {
                format!("{parent_id}/{segment}")
            };
            folders
                .entry(folder_id.clone())
                .or_insert_with(|| FolderBuilder {
                    parent_folder_id: Some(parent_id.clone()),
                    locator_prefix: folder_id.clone(),
                    display_name: (*segment).to_string(),
                    ..FolderBuilder::default()
                });
            if let Some(parent) = folders.get_mut(&parent_id) {
                if !parent.child_folder_ids.contains(&folder_id) {
                    parent.child_folder_ids.push(folder_id.clone());
                }
            }
            parent_id = folder_id;
        }
        if let Some(folder) = folders.get_mut(&parent_id) {
            folder
                .direct_asset_uuids
                .push(record.asset_uuid.to_string());
            folder.recursive_asset_count += 1;
        }
    }

    let mut ids_by_depth = folders
        .keys()
        .filter(|folder_id| folder_id.as_str() != "res://")
        .cloned()
        .collect::<Vec<_>>();
    ids_by_depth.sort_by_key(|folder_id| std::cmp::Reverse(folder_id.matches('/').count()));
    for folder_id in ids_by_depth {
        let count = folders
            .get(&folder_id)
            .map(|folder| folder.recursive_asset_count)
            .unwrap_or_default();
        let parent_id = folders
            .get(&folder_id)
            .and_then(|folder| folder.parent_folder_id.clone());
        if let Some(parent_id) = parent_id {
            if let Some(parent) = folders.get_mut(&parent_id) {
                parent.recursive_asset_count += count;
            }
        }
    }

    let folder_names = folders
        .iter()
        .map(|(id, folder)| (id.clone(), folder.display_name.clone()))
        .collect::<HashMap<_, _>>();
    let asset_names = state
        .catalog_by_uuid
        .values()
        .map(|record| (record.asset_uuid.to_string(), record.display_name.clone()))
        .collect::<HashMap<_, _>>();
    for folder in folders.values_mut() {
        folder.child_folder_ids.sort_by(|left, right| {
            folder_names[left]
                .cmp(&folder_names[right])
                .then(left.cmp(right))
        });
        folder.direct_asset_uuids.sort_by(|left, right| {
            let left_key = asset_names.get(left).cloned().unwrap_or_default();
            let right_key = asset_names.get(right).cloned().unwrap_or_default();
            left_key.cmp(&right_key).then(left.cmp(right))
        });
    }

    folders
        .into_iter()
        .map(|(folder_id, folder)| EditorAssetFolderRecord {
            folder_id,
            parent_folder_id: folder.parent_folder_id,
            locator_prefix: folder.locator_prefix,
            display_name: folder.display_name,
            child_folder_ids: folder.child_folder_ids,
            direct_asset_uuids: folder.direct_asset_uuids,
            recursive_asset_count: folder.recursive_asset_count,
        })
        .collect()
}

fn generate_preview_artifact(
    project: &ProjectManager,
    record: &AssetCatalogRecord,
    cache: &PreviewCache,
) -> Result<PathBuf, AssetImportError> {
    let key = PreviewArtifactKey::thumbnail(record.asset_uuid);
    match record.kind {
        crate::AssetKind::Texture => {
            let source_path = project.source_path_for_uri(&record.locator)?;
            let image = image::open(&source_path).map_err(|error| {
                AssetImportError::Parse(format!(
                    "failed to decode preview image {}: {error}",
                    source_path.display()
                ))
            })?;
            cache
                .write_thumbnail(&key, &image)
                .map_err(AssetImportError::from)
        }
        crate::AssetKind::Material
        | crate::AssetKind::Scene
        | crate::AssetKind::Model
        | crate::AssetKind::Shader
        | crate::AssetKind::UiLayout
        | crate::AssetKind::UiWidget
        | crate::AssetKind::UiStyle => cache
            .write_kind_placeholder(&key, preview_palette(record.kind))
            .map_err(AssetImportError::from),
    }
}

fn preview_palette(kind: crate::AssetKind) -> PreviewPalette {
    match kind {
        crate::AssetKind::Texture => PreviewPalette {
            primary: [74, 127, 173, 255],
            secondary: [35, 55, 82, 255],
            accent: [182, 220, 255, 255],
            banner: [212, 236, 255, 255],
        },
        crate::AssetKind::Material => PreviewPalette {
            primary: [156, 112, 66, 255],
            secondary: [74, 49, 28, 255],
            accent: [237, 204, 158, 255],
            banner: [247, 231, 199, 255],
        },
        crate::AssetKind::Scene => PreviewPalette {
            primary: [67, 118, 91, 255],
            secondary: [31, 60, 48, 255],
            accent: [180, 228, 200, 255],
            banner: [220, 245, 228, 255],
        },
        crate::AssetKind::Model => PreviewPalette {
            primary: [102, 97, 145, 255],
            secondary: [46, 43, 73, 255],
            accent: [210, 204, 250, 255],
            banner: [229, 225, 255, 255],
        },
        crate::AssetKind::Shader => PreviewPalette {
            primary: [170, 80, 97, 255],
            secondary: [78, 31, 43, 255],
            accent: [255, 208, 219, 255],
            banner: [255, 231, 236, 255],
        },
        crate::AssetKind::UiLayout => PreviewPalette {
            primary: [65, 112, 148, 255],
            secondary: [29, 54, 71, 255],
            accent: [190, 228, 250, 255],
            banner: [226, 243, 255, 255],
        },
        crate::AssetKind::UiWidget => PreviewPalette {
            primary: [116, 98, 169, 255],
            secondary: [52, 44, 81, 255],
            accent: [221, 210, 255, 255],
            banner: [238, 232, 255, 255],
        },
        crate::AssetKind::UiStyle => PreviewPalette {
            primary: [164, 113, 55, 255],
            secondary: [79, 53, 24, 255],
            accent: [246, 217, 173, 255],
            banner: [255, 239, 214, 255],
        },
    }
}

fn asset_record_kind(kind: crate::AssetKind) -> AssetRecordKind {
    match kind {
        crate::AssetKind::Texture => AssetRecordKind::Texture,
        crate::AssetKind::Shader => AssetRecordKind::Shader,
        crate::AssetKind::Material => AssetRecordKind::Material,
        crate::AssetKind::Scene => AssetRecordKind::Scene,
        crate::AssetKind::Model => AssetRecordKind::Model,
        crate::AssetKind::UiLayout => AssetRecordKind::UiLayout,
        crate::AssetKind::UiWidget => AssetRecordKind::UiWidget,
        crate::AssetKind::UiStyle => AssetRecordKind::UiStyle,
    }
}

fn preview_state_record(state: PreviewState) -> PreviewStateRecord {
    match state {
        PreviewState::Dirty => PreviewStateRecord::Dirty,
        PreviewState::Ready => PreviewStateRecord::Ready,
        PreviewState::Error => PreviewStateRecord::Error,
    }
}

fn preview_source_mtime(source_path: &Path) -> u64 {
    std::fs::metadata(source_path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn display_name_for_path(source_path: &Path, locator: &AssetUri) -> String {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(locator.path());
    if let Some(stripped) = file_name.strip_suffix(".toml") {
        stripped.to_string()
    } else {
        source_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(file_name)
            .to_string()
    }
}

fn display_name_for_locator(locator: &AssetUri) -> String {
    locator
        .label()
        .map(str::to_string)
        .or_else(|| {
            Path::new(locator.path())
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| locator.to_string())
}

fn meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    path.with_file_name(format!("{file_name}.meta.toml"))
}

fn direct_references(imported: &ImportedAsset) -> Vec<AssetReference> {
    let mut references = Vec::new();
    match imported {
        ImportedAsset::Material(material) => {
            references.push(material.shader.clone());
            references.extend(
                [
                    material.base_color_texture.clone(),
                    material.normal_texture.clone(),
                    material.metallic_roughness_texture.clone(),
                    material.occlusion_texture.clone(),
                    material.emissive_texture.clone(),
                ]
                .into_iter()
                .flatten(),
            );
        }
        ImportedAsset::Scene(scene) => {
            for entity in &scene.entities {
                if let Some(mesh) = &entity.mesh {
                    references.push(mesh.model.clone());
                    references.push(mesh.material.clone());
                }
            }
        }
        ImportedAsset::UiLayout(asset) => {
            references.extend(crate::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::UiWidget(asset) => {
            references.extend(crate::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::UiStyle(asset) => {
            references.extend(crate::assets::ui_asset_references(&asset.document));
        }
        ImportedAsset::Texture(_) | ImportedAsset::Shader(_) | ImportedAsset::Model(_) => {}
    }

    dedup_references(references)
}

fn dedup_references(references: Vec<AssetReference>) -> Vec<AssetReference> {
    let mut seen = HashMap::<ResourceId, AssetReference>::new();
    for reference in references {
        let id = ResourceId::from_asset_uuid_label(reference.uuid, reference.locator.label());
        seen.entry(id).or_insert(reference);
    }
    let mut deduped = seen.into_values().collect::<Vec<_>>();
    deduped.sort_by(|left, right| left.locator.to_string().cmp(&right.locator.to_string()));
    deduped
}
