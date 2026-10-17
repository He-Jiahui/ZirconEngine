use super::asset_error_message;
use crate::asset::{AssetId, AssetUri, ShaderAsset};
use crate::core::resource::{ResourceManager, RuntimeResourceState};
use crate::core::CoreError;
use std::sync::Arc;
/// The source owns bounded residency requests; browser implementations enqueue Fetch work.
/// An already resident payload is returned through ResourceSnapshot/ResourceLease below.
pub trait CookedAssetSource: Send + Sync {
    fn ensure_resident(&self, resources: &ResourceManager, id: AssetId) -> Result<(), CoreError>;
    fn close(&self);
    fn render_artifact_store(&self) -> Option<crate::asset::artifact::RenderArtifactStore>;
}
#[derive(Clone)]
pub struct RuntimeAssetManager {
    resources: ResourceManager,
    pub(super) source: Arc<dyn CookedAssetSource>,
    pub(super) generation: super::RuntimeAssetGenerationToken,
}
impl std::fmt::Debug for RuntimeAssetManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeAssetManager")
            .finish_non_exhaustive()
    }
}
impl RuntimeAssetManager {
    pub fn new(resources: ResourceManager, source: Arc<dyn CookedAssetSource>) -> Self {
        Self {
            resources,
            source,
            generation: super::RuntimeAssetGenerationToken(Arc::new(std::sync::Mutex::new(true))),
        }
    }
    pub fn resource_manager(&self) -> ResourceManager {
        self.resources.clone()
    }
    pub(crate) fn ensure_resident(&self, id: AssetId) -> Result<(), CoreError> {
        if self.resources.get_untyped(id).is_some() {
            return Ok(());
        }
        self.source.ensure_resident(&self.resources, id)
    }
    pub fn close(&self) {
        *self.generation.0.lock().unwrap_or_else(|e| e.into_inner()) = false;
        self.source.close();
    }
    pub fn resolve_asset_id(&self, uri: &AssetUri) -> Option<AssetId> {
        self.resources
            .registry()
            .get_by_locator(uri)
            .map(|record| record.id)
    }
    pub fn load_shader_asset_by_uri(&self, uri: &AssetUri) -> Result<ShaderAsset, CoreError> {
        self.load_shader_asset(
            self.resolve_asset_id(uri)
                .ok_or_else(|| asset_error_message(format!("missing shader locator {uri}")))?,
        )
    }
    pub fn runtime_ref_count(&self, id: AssetId) -> Option<usize> {
        self.resources.ref_count(id)
    }
    pub fn runtime_resource_state(&self, id: AssetId) -> Option<RuntimeResourceState> {
        self.resources.runtime_state(id)
    }
    pub(in crate::asset) fn catalog_generation_is_ready(&self) -> bool {
        *self.generation.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}
