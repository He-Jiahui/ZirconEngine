//! Module wiring and high-level asset manager service.

use crossbeam_channel::unbounded;
use std::fmt;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};

use zircon_core::{
    ChannelReceiver, ChannelSender, CoreError, DriverDescriptor, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};

use zircon_manager::{
    AssetChangeKind as FacadeAssetChangeKind, AssetChangeRecord,
    AssetManager as AssetManagerFacade, AssetManagerHandle, AssetPipelineInfo, AssetRecordKind,
    AssetStatusRecord, EditorAssetManagerHandle, ProjectInfo, ResourceChangeKind,
    ResourceChangeRecord, ResourceManager as ResourceManagerFacade, ResourceManagerHandle,
    ResourceStateRecord, ResourceStatusRecord,
};
use zircon_module::{dependency_on, factory, qualified_name};

use crate::{
    load::mesh::generate_cube_mesh, AlphaMode, AssetId, AssetKind, AssetMetadata,
    AssetReference, AssetUri, AssetWatcher,
    DefaultEditorAssetManager as EditorAssetManagerService, ImportedAsset, MaterialAsset,
    MaterialMarker, ModelAsset, ModelMarker, ModelPrimitiveAsset, ProjectManager, ResourceHandle,
    ResourceLease, ResourceManager, RuntimeResourceState, SceneAsset, SceneMarker, ShaderAsset,
    ShaderMarker, TextureAsset, TextureMarker,
};

pub const ASSET_MODULE_NAME: &str = "AssetModule";
pub const ASSET_IO_DRIVER_NAME: &str = "AssetModule.Driver.AssetIoDriver";
pub const PROJECT_ASSET_MANAGER_NAME: &str = "AssetModule.Manager.ProjectAssetManager";
pub const ASSET_MANAGER_NAME: &str = zircon_manager::ASSET_MANAGER_NAME;
pub const RESOURCE_MANAGER_NAME: &str = zircon_manager::RESOURCE_MANAGER_NAME;
pub const DEFAULT_EDITOR_ASSET_MANAGER_NAME: &str =
    "AssetModule.Manager.DefaultEditorAssetManager";
pub const EDITOR_ASSET_MANAGER_NAME: &str = zircon_manager::EDITOR_ASSET_MANAGER_NAME;

#[derive(Clone, Debug, Default)]
pub struct AssetIoDriver;

#[derive(Clone)]
pub struct ProjectAssetManager {
    default_worker_count: usize,
    project: Arc<RwLock<Option<ProjectManager>>>,
    resource_manager: ResourceManager,
    editor_asset_manager: Arc<EditorAssetManagerService>,
    change_subscribers: Arc<Mutex<Vec<ChannelSender<AssetChangeRecord>>>>,
    watcher: Arc<Mutex<Option<AssetWatcher>>>,
}

impl fmt::Debug for ProjectAssetManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectAssetManager")
            .field("default_worker_count", &self.default_worker_count)
            .finish_non_exhaustive()
    }
}

impl Default for ProjectAssetManager {
    fn default() -> Self {
        Self {
            default_worker_count: std::thread::available_parallelism()
                .map_or(2, |n| n.get().max(2) - 1),
            project: Arc::new(RwLock::new(None)),
            resource_manager: resource_manager_with_builtins(),
            editor_asset_manager: Arc::new(EditorAssetManagerService::default()),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }
}

impl ProjectAssetManager {
    pub fn new(default_worker_count: usize) -> Self {
        Self {
            default_worker_count: default_worker_count.max(1),
            project: Arc::new(RwLock::new(None)),
            resource_manager: resource_manager_with_builtins(),
            editor_asset_manager: Arc::new(EditorAssetManagerService::default()),
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_editor_asset_manager(
        default_worker_count: usize,
        editor_asset_manager: Arc<EditorAssetManagerService>,
    ) -> Self {
        Self {
            default_worker_count: default_worker_count.max(1),
            project: Arc::new(RwLock::new(None)),
            resource_manager: resource_manager_with_builtins(),
            editor_asset_manager,
            change_subscribers: Arc::new(Mutex::new(Vec::new())),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    pub fn spawn_worker_pool(
        &self,
    ) -> Result<crate::worker_pool::AssetWorkerPool, zircon_core::ZirconError> {
        crate::worker_pool::AssetWorkerPool::new(self.default_worker_count)
    }

    pub fn default_worker_count(&self) -> usize {
        self.default_worker_count
    }

    pub fn resource_manager(&self) -> ResourceManager {
        self.resource_manager.clone()
    }

    pub fn editor_asset_manager(&self) -> Arc<EditorAssetManagerService> {
        self.editor_asset_manager.clone()
    }

    pub fn resolve_asset_id(&self, uri: &AssetUri) -> Option<AssetId> {
        self.resource_manager()
            .registry()
            .get_by_locator(uri)
            .map(|record| record.id())
    }

    pub fn load_imported_asset(&self, id: AssetId) -> Result<ImportedAsset, CoreError> {
        let kind = self
            .resource_manager()
            .registry()
            .get(id)
            .map(|record| record.kind)
            .ok_or_else(|| asset_error_message(format!("missing resource record for asset id {id}")))?;

        match kind {
            AssetKind::Model => self.load_model_asset(id).map(ImportedAsset::Model),
            AssetKind::Material => self.load_material_asset(id).map(ImportedAsset::Material),
            AssetKind::Texture => self.load_texture_asset(id).map(ImportedAsset::Texture),
            AssetKind::Shader => self.load_shader_asset(id).map(ImportedAsset::Shader),
            AssetKind::Scene => self.load_scene_asset(id).map(ImportedAsset::Scene),
        }
    }

    pub fn load_model_asset(&self, id: AssetId) -> Result<ModelAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn load_material_asset(&self, id: AssetId) -> Result<MaterialAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
    }

    pub fn load_texture_asset(&self, id: AssetId) -> Result<TextureAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn load_shader_asset(&self, id: AssetId) -> Result<ShaderAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn load_scene_asset(&self, id: AssetId) -> Result<SceneAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
    }

    pub fn acquire_model_asset(&self, id: AssetId) -> Result<ResourceLease<ModelAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn acquire_material_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<MaterialAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
    }

    pub fn acquire_texture_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<TextureAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn acquire_shader_asset(&self, id: AssetId) -> Result<ResourceLease<ShaderAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn acquire_scene_asset(&self, id: AssetId) -> Result<ResourceLease<SceneAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
    }

    pub fn runtime_ref_count(&self, id: AssetId) -> Option<usize> {
        self.resource_manager().ref_count(id)
    }

    pub fn runtime_resource_state(&self, id: AssetId) -> Option<RuntimeResourceState> {
        self.resource_manager().runtime_state(id)
    }

    pub fn load_shader_asset_by_uri(&self, uri: &AssetUri) -> Result<ShaderAsset, CoreError> {
        let id = self
            .resolve_asset_id(uri)
            .ok_or_else(|| asset_error_message(format!("missing shader locator {uri}")))?;
        self.load_shader_asset(id)
    }

    fn project_read(&self) -> std::sync::RwLockReadGuard<'_, Option<ProjectManager>> {
        self.project.read().expect("asset project lock poisoned")
    }

    fn project_write(&self) -> std::sync::RwLockWriteGuard<'_, Option<ProjectManager>> {
        self.project.write().expect("asset project lock poisoned")
    }

    fn broadcast(&self, changes: Vec<AssetChangeRecord>) {
        if changes.is_empty() {
            return;
        }

        let mut subscribers = self
            .change_subscribers
            .lock()
            .expect("asset subscribers lock poisoned");
        subscribers.retain(|sender| {
            changes
                .iter()
                .all(|change| sender.send(change.clone()).is_ok())
        });
    }

    fn load_typed<TMarker, TAsset>(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<TAsset, CoreError>
    where
        TMarker: crate::ResourceMarker,
        TAsset: crate::ResourceData + Clone,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .get::<TMarker, TAsset>(handle)
            .map(|asset| asset.as_ref().clone())
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }

    fn acquire_typed<TMarker, TAsset>(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<ResourceLease<TAsset>, CoreError>
    where
        TMarker: crate::ResourceMarker,
        TAsset: crate::ResourceData,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .acquire::<TMarker, TAsset>(handle)
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }

    fn restart_watcher(&self) -> Result<(), CoreError> {
        let assets_root = {
            let project = self.project_read();
            let project = project
                .as_ref()
                .ok_or_else(|| asset_error_message("no project is currently open"))?;
            project.paths().assets_root().to_path_buf()
        };
        let manager = self.clone();
        let watcher = AssetWatcher::spawn(assets_root, move |changes| {
            manager.process_watch_changes(changes);
        })
        .map_err(asset_error)?;
        *self.watcher.lock().expect("asset watcher lock poisoned") = Some(watcher);
        Ok(())
    }

    fn ensure_resident(&self, id: AssetId) -> Result<(), CoreError> {
        if self.resource_manager().get_untyped(id).is_some() {
            return Ok(());
        }

        let metadata = self
            .resource_manager()
            .registry()
            .get(id)
            .cloned()
            .ok_or_else(|| asset_error_message(format!("missing resource record for asset id {id}")))?;
        let imported = match metadata.primary_locator.scheme() {
            crate::AssetUriScheme::Builtin => builtin_resources()
                .into_iter()
                .find_map(|(locator_text, asset)| {
                    let locator = AssetUri::parse(locator_text).ok()?;
                    (locator == metadata.primary_locator).then_some(asset)
                })
                .ok_or_else(|| {
                    asset_error_message(format!(
                        "missing builtin runtime payload for {}",
                        metadata.primary_locator
                    ))
                })?,
            crate::AssetUriScheme::Res | crate::AssetUriScheme::Library => {
                let project = self.project_read();
                let project = project
                    .as_ref()
                    .ok_or_else(|| asset_error_message("no project is currently open"))?;
                project.load_artifact_by_id(id).map_err(asset_error)?
            }
            crate::AssetUriScheme::Memory => {
                return Err(asset_error_message(format!(
                    "memory resource {id} cannot be restored by ProjectAssetManager"
                )));
            }
        };
        store_runtime_payload(&self.resource_manager, id, imported);
        Ok(())
    }

    fn sync_project_resources(&self, project: &ProjectManager) -> Result<(), CoreError> {
        for metadata in project.registry().values() {
            let imported = project.load_artifact_by_id(metadata.id()).map_err(asset_error)?;
            register_project_resource(&self.resource_manager, metadata.clone(), imported);
        }
        Ok(())
    }

    fn process_watch_changes(&self, changes: Vec<crate::AssetChange>) {
        if changes.is_empty() {
            return;
        }

        {
            let mut project = self.project_write();
            let Some(project) = project.as_mut() else {
                return;
            };
            if project.scan_and_import().is_err() || self.sync_project_resources(project).is_err() {
                return;
            }
            let _ = self.editor_asset_manager.sync_from_project(project.clone());
        }

        self.broadcast(
            changes
                .into_iter()
                .map(|change| AssetChangeRecord {
                    kind: match change.kind {
                        crate::AssetChangeKind::Added => FacadeAssetChangeKind::Added,
                        crate::AssetChangeKind::Modified => FacadeAssetChangeKind::Modified,
                        crate::AssetChangeKind::Removed => FacadeAssetChangeKind::Removed,
                        crate::AssetChangeKind::Renamed => FacadeAssetChangeKind::Renamed,
                    },
                    uri: change.uri.to_string(),
                    previous_uri: change.previous_uri.map(|uri| uri.to_string()),
                })
                .collect(),
        );
    }
}

impl AssetManagerFacade for ProjectAssetManager {
    fn pipeline_info(&self) -> AssetPipelineInfo {
        AssetPipelineInfo {
            default_worker_count: self.default_worker_count(),
        }
    }

    fn open_project(&self, root_path: &str) -> Result<ProjectInfo, CoreError> {
        let mut project = ProjectManager::open(root_path).map_err(asset_error)?;
        let previous_locators = self
            .project_read()
            .as_ref()
            .map(project_locators)
            .unwrap_or_default();
        let imported = project.scan_and_import().map_err(asset_error)?;
        clear_removed_project_resources(&self.resource_manager, &previous_locators, &project);
        self.sync_project_resources(&project)?;
        let info = project_info(&project);
        self.editor_asset_manager
            .sync_from_project(project.clone())
            .map_err(asset_error)?;
        *self.project_write() = Some(project);
        self.restart_watcher()?;
        self.broadcast(
            imported
                .into_iter()
                .map(|metadata| AssetChangeRecord {
                    kind: FacadeAssetChangeKind::Added,
                    uri: metadata.primary_locator().to_string(),
                    previous_uri: None,
                })
                .collect(),
        );
        Ok(info)
    }

    fn current_project(&self) -> Option<ProjectInfo> {
        self.project_read().as_ref().map(project_info)
    }

    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord> {
        let uri = AssetUri::parse(uri).ok()?;
        let project = self.project_read();
        let project = project.as_ref()?;
        project.registry().get_by_locator(&uri).map(status_record)
    }

    fn list_assets(&self) -> Vec<AssetStatusRecord> {
        let project = self.project_read();
        let Some(project) = project.as_ref() else {
            return Vec::new();
        };
        let mut assets = project
            .registry()
            .values()
            .map(status_record)
            .collect::<Vec<_>>();
        assets.sort_by(|left, right| left.uri.cmp(&right.uri));
        assets
    }

    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChangeRecord> {
        let (sender, receiver) = unbounded();
        self.change_subscribers
            .lock()
            .expect("asset subscribers lock poisoned")
            .push(sender);
        receiver
    }

    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError> {
        let uri = AssetUri::parse(uri).map_err(asset_error)?;
        let mut project = self.project_write();
        let Some(project) = project.as_mut() else {
            return Ok(None);
        };
        project.scan_and_import().map_err(asset_error)?;
        self.sync_project_resources(project)?;
        self.editor_asset_manager
            .sync_from_project(project.clone())
            .map_err(asset_error)?;
        let status = project.registry().get_by_locator(&uri).map(status_record);
        if let Some(status) = status.clone() {
            self.broadcast(vec![AssetChangeRecord {
                kind: FacadeAssetChangeKind::Modified,
                uri: status.uri.clone(),
                previous_uri: None,
            }]);
        }
        Ok(status)
    }

    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let mut project = self.project_write();
        let Some(project) = project.as_mut() else {
            return Ok(Vec::new());
        };
        let imported = project.scan_and_import().map_err(asset_error)?;
        self.sync_project_resources(project)?;
        self.editor_asset_manager
            .sync_from_project(project.clone())
            .map_err(asset_error)?;
        let statuses = imported.iter().map(status_record).collect::<Vec<_>>();
        self.broadcast(
            imported
                .into_iter()
                .map(|metadata| AssetChangeRecord {
                    kind: FacadeAssetChangeKind::Modified,
                    uri: metadata.primary_locator().to_string(),
                    previous_uri: None,
                })
                .collect(),
        );
        Ok(statuses)
    }
}

impl ResourceManagerFacade for ProjectAssetManager {
    fn resolve_resource_id(&self, locator: &str) -> Option<String> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.id().to_string())
    }

    fn resource_status(&self, locator: &str) -> Option<ResourceStatusRecord> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(resource_status_record)
    }

    fn list_resources(&self) -> Vec<ResourceStatusRecord> {
        let mut resources = self
            .resource_manager()
            .registry()
            .values()
            .map(resource_status_record)
            .collect::<Vec<_>>();
        resources.sort_by(|left, right| left.locator.cmp(&right.locator));
        resources
    }

    fn resource_revision(&self, locator: &str) -> Option<u64> {
        let locator = AssetUri::parse(locator).ok()?;
        self.resource_manager()
            .registry()
            .get_by_locator(&locator)
            .map(|record| record.revision)
    }

    fn subscribe_resource_changes(&self) -> ChannelReceiver<ResourceChangeRecord> {
        let source = self.resource_manager().subscribe();
        let (sender, receiver) = unbounded();
        std::thread::Builder::new()
            .name("zircon-resource-event-bridge".to_string())
            .spawn(move || {
                while let Ok(event) = source.recv() {
                    if sender.send(resource_change_record(event)).is_err() {
                        break;
                    }
                }
            })
            .expect("resource event bridge thread");
        receiver
    }
}

fn asset_error(error: impl std::error::Error) -> CoreError {
    CoreError::Initialization(PROJECT_ASSET_MANAGER_NAME.to_string(), error.to_string())
}

fn asset_error_message(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(PROJECT_ASSET_MANAGER_NAME.to_string(), message.into())
}

fn project_info(project: &ProjectManager) -> ProjectInfo {
    ProjectInfo {
        root_path: project.paths().root().to_string_lossy().into_owned(),
        name: project.manifest().name.clone(),
        default_scene_uri: project.manifest().default_scene.to_string(),
        library_version: project.manifest().library_version,
    }
}

fn status_record(metadata: &AssetMetadata) -> AssetStatusRecord {
    AssetStatusRecord {
        id: metadata.id().to_string(),
        uri: metadata.primary_locator().to_string(),
        kind: match metadata.kind {
            AssetKind::Texture => AssetRecordKind::Texture,
            AssetKind::Shader => AssetRecordKind::Shader,
            AssetKind::Material => AssetRecordKind::Material,
            AssetKind::Scene => AssetRecordKind::Scene,
            AssetKind::Model => AssetRecordKind::Model,
        },
        artifact_uri: metadata.artifact_locator().map(ToString::to_string),
        imported: metadata.artifact_locator().is_some(),
        source_hash: metadata.source_hash.clone(),
        importer_version: metadata.importer_version,
        config_hash: metadata.config_hash.clone(),
    }
}

fn resource_status_record(metadata: &AssetMetadata) -> ResourceStatusRecord {
    ResourceStatusRecord {
        id: metadata.id().to_string(),
        locator: metadata.primary_locator().to_string(),
        kind: match metadata.kind {
            AssetKind::Texture => AssetRecordKind::Texture,
            AssetKind::Shader => AssetRecordKind::Shader,
            AssetKind::Material => AssetRecordKind::Material,
            AssetKind::Scene => AssetRecordKind::Scene,
            AssetKind::Model => AssetRecordKind::Model,
        },
        artifact_locator: metadata.artifact_locator().map(ToString::to_string),
        revision: metadata.revision,
        state: match metadata.state {
            crate::ResourceState::Pending => ResourceStateRecord::Pending,
            crate::ResourceState::Ready => ResourceStateRecord::Ready,
            crate::ResourceState::Error => ResourceStateRecord::Error,
            crate::ResourceState::Reloading => ResourceStateRecord::Reloading,
        },
        dependency_ids: metadata
            .dependency_ids
            .iter()
            .map(ToString::to_string)
            .collect(),
        diagnostics: metadata
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect(),
    }
}

fn resource_change_record(event: crate::ResourceEvent) -> ResourceChangeRecord {
    ResourceChangeRecord {
        kind: match event.kind {
            crate::ResourceEventKind::Added => ResourceChangeKind::Added,
            crate::ResourceEventKind::Updated => ResourceChangeKind::Updated,
            crate::ResourceEventKind::Removed => ResourceChangeKind::Removed,
            crate::ResourceEventKind::Renamed => ResourceChangeKind::Renamed,
            crate::ResourceEventKind::ReloadFailed => ResourceChangeKind::ReloadFailed,
        },
        id: event.id.to_string(),
        locator: event.locator.map(|locator| locator.to_string()),
        previous_locator: event.previous_locator.map(|locator| locator.to_string()),
        revision: event.revision,
    }
}

fn project_locators(project: &ProjectManager) -> HashSet<AssetUri> {
    project
        .registry()
        .values()
        .map(|metadata| metadata.primary_locator().clone())
        .collect()
}

fn clear_removed_project_resources(
    resource_manager: &ResourceManager,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) {
    let current = project_locators(project);
    for locator in previous_locators.difference(&current) {
        let _ = resource_manager.remove_by_locator(locator);
    }
}

fn register_project_resource(
    resource_manager: &ResourceManager,
    metadata: AssetMetadata,
    imported: ImportedAsset,
) {
    match imported {
        ImportedAsset::Texture(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Shader(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Material(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Scene(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
        ImportedAsset::Model(asset) => {
            resource_manager.register_ready(metadata, asset);
        }
    }
}

fn store_runtime_payload(resource_manager: &ResourceManager, id: AssetId, imported: ImportedAsset) {
    match imported {
        ImportedAsset::Texture(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Shader(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Material(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Scene(asset) => {
            resource_manager.store_payload(id, asset);
        }
        ImportedAsset::Model(asset) => {
            resource_manager.store_payload(id, asset);
        }
    }
}

fn resource_manager_with_builtins() -> ResourceManager {
    let manager = ResourceManager::new();

    for (locator_text, asset) in builtin_resources() {
        let locator = AssetUri::parse(locator_text).expect("builtin locator");
        let kind = match &asset {
            ImportedAsset::Texture(_) => AssetKind::Texture,
            ImportedAsset::Shader(_) => AssetKind::Shader,
            ImportedAsset::Material(_) => AssetKind::Material,
            ImportedAsset::Scene(_) => AssetKind::Scene,
            ImportedAsset::Model(_) => AssetKind::Model,
        };
        let record = AssetMetadata::new(AssetId::from_locator(&locator), kind, locator);
        register_project_resource(&manager, record, asset);
    }

    manager
}

fn builtin_resources() -> Vec<(&'static str, ImportedAsset)> {
    let mesh = generate_cube_mesh();
    vec![
        (
            "builtin://cube",
            ImportedAsset::Model(ModelAsset {
                uri: AssetUri::parse("builtin://cube").expect("builtin cube uri"),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: mesh.vertices.clone(),
                    indices: mesh.indices.clone(),
                }],
            }),
        ),
        (
            "builtin://missing-model",
            ImportedAsset::Model(ModelAsset {
                uri: AssetUri::parse("builtin://missing-model").expect("missing model uri"),
                primitives: vec![ModelPrimitiveAsset {
                    vertices: mesh.vertices,
                    indices: mesh.indices,
                }],
            }),
        ),
        (
            "builtin://material/default",
            ImportedAsset::Material(MaterialAsset {
                name: Some("Builtin Default".to_string()),
                shader: builtin_reference("builtin://shader/pbr.wgsl"),
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.0,
                roughness: 1.0,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.0, 0.0, 0.0],
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
            }),
        ),
        (
            "builtin://missing-material",
            ImportedAsset::Material(MaterialAsset {
                name: Some("Builtin Missing".to_string()),
                shader: builtin_reference("builtin://shader/pbr.wgsl"),
                base_color: [1.0, 0.0, 1.0, 1.0],
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.0,
                roughness: 1.0,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.0, 0.0, 0.0],
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
            }),
        ),
        (
            "builtin://shader/pbr.wgsl",
            ImportedAsset::Shader(ShaderAsset {
                uri: AssetUri::parse("builtin://shader/pbr.wgsl").expect("builtin shader uri"),
                source: builtin_pbr_wgsl().to_string(),
            }),
        ),
    ]
}

fn builtin_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).expect("builtin asset reference"))
}

pub(crate) fn builtin_pbr_wgsl() -> &'static str {
    r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.position = scene.view_proj * world_position;
    out.world_normal = normalize((model.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(color_texture, color_sampler, input.uv) * model.tint;
    let ndotl = max(dot(normalize(input.world_normal), normalize(-scene.light_dir.xyz)), 0.0);
    let lighting = 0.15 + ndotl;
    return vec4<f32>(albedo.rgb * scene.light_color.rgb * lighting, albedo.a);
}
"#
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        ASSET_MODULE_NAME,
        "Asynchronous asset I/O and CPU-side decoding",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Driver, "AssetIoDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(AssetIoDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultEditorAssetManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(EditorAssetManagerService::default()) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Manager, "ProjectAssetManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultEditorAssetManager",
        )],
        factory(|core| {
            let editor_asset_manager =
                core.resolve_manager::<EditorAssetManagerService>(DEFAULT_EDITOR_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(ProjectAssetManager::with_editor_asset_manager(
                std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1),
                editor_asset_manager,
            )) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Manager, "AssetManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let manager = core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(AssetManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Manager, "ResourceManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "ProjectAssetManager",
        )],
        factory(|core| {
            let manager = core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(ResourceManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(ASSET_MODULE_NAME, ServiceKind::Manager, "EditorAssetManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            ASSET_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultEditorAssetManager",
        )],
        factory(|core| {
            let manager =
                core.resolve_manager::<EditorAssetManagerService>(DEFAULT_EDITOR_ASSET_MANAGER_NAME)?;
            Ok(Arc::new(EditorAssetManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}
