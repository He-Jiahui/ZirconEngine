//! Read-only, bounded browser resource residency over the desktop Raw export.
use super::async_value::AsyncValue;
use super::{asset_error_message, CookedAssetSource, RuntimeAssetManager};
use crate::asset::artifact::{
    RawAssetCatalog, RawSceneCatalogEntry, RenderArtifactStore, RAW_CATALOG_MAX_BYTES,
};
use crate::asset::{AssetId, AssetKind};
use crate::core::resource::{
    ResourceManager, ResourceMutationBatch, ResourceRecord, ResourceState,
};
use crate::core::runtime::{TaskGraphScope, TaskGraphScopeDescriptor};
use crate::core::{CoreError, CoreHandle};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
mod request;

#[derive(Clone, Copy, Debug)]
pub struct BrowserAssetLimits {
    pub max_catalog_entries: usize,
    pub max_in_flight: usize,
    pub max_payload_bytes: u64,
    /// Sum of exported payload sizes admitted for resident and loading CPU resources.
    /// Exact GPU/decoded-block accounting remains owned by the render residency managers.
    pub max_resident_payload_bytes: u64,
}
impl Default for BrowserAssetLimits {
    fn default() -> Self {
        Self {
            max_catalog_entries: 4096,
            max_in_flight: 32,
            max_payload_bytes: 32 * 1024 * 1024,
            max_resident_payload_bytes: 128 * 1024 * 1024,
        }
    }
}
#[derive(Clone)]
struct Payload {
    kind: AssetKind,
    revision: u64,
    bytes: u64,
    hash: [u8; 32],
}
struct Request {
    done: Arc<AsyncValue<()>>,
    task: Mutex<Option<crate::core::runtime::TaskHandle>>,
}
struct State {
    accepting: bool,
    next_task: u64,
    requests: HashMap<AssetId, Arc<Request>>,
}
pub struct BrowserAssetSource {
    root: Arc<str>,
    resources: ResourceManager,
    payloads: HashMap<AssetId, Payload>,
    scope: TaskGraphScope,
    limits: BrowserAssetLimits,
    state: Mutex<State>,
    scene: Option<RawSceneCatalogEntry>,
    startup_assets: Box<[AssetId]>,
    ui_roots: Box<[AssetId]>,
}
impl BrowserAssetSource {
    pub async fn open(
        core: &CoreHandle,
        root: impl Into<Arc<str>>,
        limits: BrowserAssetLimits,
    ) -> Result<Arc<Self>, CoreError> {
        if limits.max_catalog_entries == 0
            || limits.max_in_flight == 0
            || limits.max_payload_bytes == 0
            || limits.max_resident_payload_bytes < limits.max_payload_bytes
        {
            return Err(asset_error_message("invalid browser asset limits"));
        }
        let root = root.into();
        let scope = core
            .task_graph()
            .create_scope(
                TaskGraphScopeDescriptor::new("browser-cooked-assets")
                    .with_task_capacity(limits.max_in_flight),
            )
            .map_err(|e| asset_error_message(e.to_string()))?;
        let bytes = request::fetch_catalog(&scope, Arc::clone(&root)).await?;
        let catalog: RawAssetCatalog =
            serde_json::from_slice(&bytes).map_err(|e| asset_error_message(e.to_string()))?;
        drop(bytes);
        if catalog.schema_version != 1
            || catalog.target_platform != "wasm32-webgpu"
            || catalog.assets.len() > limits.max_catalog_entries
        {
            return Err(asset_error_message(
                "unsupported browser asset catalog schema, target, or entry count",
            ));
        }
        if catalog.startup_assets.len() > limits.max_catalog_entries
            || catalog.ui_roots.len() > 8
            || catalog.scene.as_ref().is_some_and(|scene| {
                scene.payload_bytes == 0 || scene.payload_bytes > RAW_CATALOG_MAX_BYTES
            })
        {
            return Err(asset_error_message(
                "browser startup metadata exceeds its limits",
            ));
        }
        let resources = ResourceManager::new();
        let mut payloads = HashMap::with_capacity(catalog.assets.len());
        let mut locators = HashSet::with_capacity(catalog.assets.len());
        let mut batch = ResourceMutationBatch::new();
        for entry in catalog.assets {
            if entry.payload_bytes < 8
                || entry.payload_bytes > limits.max_payload_bytes
                || entry.record.state != ResourceState::Ready
                || payloads.contains_key(&entry.id())
                || !locators.insert(entry.record.primary_locator.clone())
            {
                return Err(asset_error_message(
                    "invalid, oversized, or duplicate browser asset catalog entry",
                ));
            }
            payloads.insert(
                entry.id(),
                Payload {
                    kind: entry.record.kind,
                    revision: 0,
                    bytes: entry.payload_bytes,
                    hash: entry.content_hash,
                },
            );
            batch = batch.upsert_lazy(entry.record);
        }
        resources
            .commit(batch)
            .map_err(|e| asset_error_message(e.to_string()))?;
        // Resource authority owns its local revisions; exported hashes remain immutable source identity.
        for (id, payload) in &mut payloads {
            payload.revision = resources
                .registry()
                .get(*id)
                .ok_or_else(|| asset_error_message("catalog registration missing"))?
                .revision;
        }
        for id in catalog.startup_assets.iter().chain(&catalog.ui_roots) {
            if !payloads.contains_key(id) {
                return Err(asset_error_message(
                    "startup resource is missing from the browser catalog",
                ));
            }
        }
        for id in payloads.keys() {
            let registry = resources.registry();
            let record = registry
                .get(*id)
                .ok_or_else(|| asset_error_message("catalog entry disappeared"))?;
            if record
                .dependency_ids
                .iter()
                .any(|id| !payloads.contains_key(id))
            {
                return Err(asset_error_message(
                    "browser catalog has a missing resource dependency",
                ));
            }
        }
        Ok(Arc::new(Self {
            root,
            resources,
            payloads,
            scope,
            limits,
            scene: catalog.scene,
            startup_assets: catalog.startup_assets.into_boxed_slice(),
            ui_roots: catalog.ui_roots.into_boxed_slice(),
            state: Mutex::new(State {
                accepting: true,
                next_task: 1,
                requests: HashMap::new(),
            }),
        }))
    }

    pub fn ui_roots(&self) -> &[AssetId] {
        &self.ui_roots
    }
    /// Only the selected roots and their dependency closure become resident.
    pub fn startup_asset_ids(&self) -> Vec<AssetId> {
        let registry = self.resources.registry();
        let mut seen = HashSet::new();
        let mut pending = self
            .startup_assets
            .iter()
            .chain(self.ui_roots.iter())
            .copied()
            .collect::<Vec<_>>();
        let mut result = Vec::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            if let Some(record) = registry.get(id) {
                pending.extend(record.dependency_ids.iter().copied());
            }
            result.push(id);
        }
        result
    }
    pub async fn load_scene(&self) -> Result<crate::scene::DynamicScene, CoreError> {
        let scene = self
            .scene
            .as_ref()
            .ok_or_else(|| asset_error_message("browser bundle has no startup scene"))?;
        let id = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if !state.accepting {
                return Err(asset_error_message("browser asset source is closed"));
            }
            let id = state.next_task;
            state.next_task = id
                .checked_add(1)
                .ok_or_else(|| asset_error_message("browser task sequence exhausted"))?;
            id
        };
        let bytes = request::fetch_metadata(
            &self.scope,
            id,
            format!(
                "{}/{}",
                self.root.trim_end_matches('/'),
                scene.relative_payload_path()
            ),
            Some(scene.payload_bytes),
            RAW_CATALOG_MAX_BYTES,
        )
        .await?;
        if blake3::hash(&bytes).as_bytes() != &scene.content_hash {
            return Err(asset_error_message("browser scene content hash mismatch"));
        }
        serde_json::from_slice(&bytes).map_err(|e| asset_error_message(e.to_string()))
    }

    pub fn asset_ids(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.payloads.keys().copied()
    }
    pub fn resource_manager(&self) -> ResourceManager {
        self.resources.clone()
    }
    pub fn manager(self: &Arc<Self>) -> RuntimeAssetManager {
        RuntimeAssetManager::new(
            self.resources.clone(),
            Arc::clone(self) as Arc<dyn CookedAssetSource>,
        )
    }
    pub async fn preload(self: &Arc<Self>, id: AssetId) -> Result<(), CoreError> {
        if self.resources.get_untyped(id).is_some() {
            return Ok(());
        }
        let request = self.request(id)?;
        request
            .done
            .wait()
            .map_err(|e| asset_error_message(e.to_string()))?
            .await
            .map_err(|e| asset_error_message(e.to_string()))
    }
    pub fn close(&self) {
        self.close_source();
    }
    fn close_source(&self) {
        let tasks = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.accepting = false;
            state
                .requests
                .values()
                .filter_map(|request| {
                    request
                        .task
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .take()
                })
                .collect::<Vec<_>>()
        };
        for task in tasks {
            task.request_cancellation();
        }
        self.scope.close_admission();
    }
    pub fn pending_requests(&self) -> usize {
        self.scope
            .census()
            .queued
            .saturating_add(self.scope.census().running)
    }
}
impl CookedAssetSource for BrowserAssetSource {
    fn ensure_resident(&self, _: &ResourceManager, id: AssetId) -> Result<(), CoreError> {
        if self.resources.get_untyped(id).is_some() {
            return Ok(());
        }
        let request = self.request(id)?;
        match request.done.result() {
            Some(Err(error)) => Err(asset_error_message(error.to_string())),
            _ => Err(asset_error_message(format!(
                "cooked resource {id} is loading asynchronously"
            ))),
        }
    }
    fn close(&self) {
        self.close_source();
    }
    fn render_artifact_store(&self) -> Option<RenderArtifactStore> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .accepting
            .then(|| RenderArtifactStore::from_url(Arc::clone(&self.root)))
    }
}
impl Drop for BrowserAssetSource {
    fn drop(&mut self) {
        self.close_source();
    }
}
