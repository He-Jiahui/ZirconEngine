use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState,
};
use zircon_runtime::graphics::RuntimePrepareCollectorContext;

use crate::hybrid_gi::scene_representation::HybridGiGlobalSdfClipmapBounds;

use super::material_capture::RuntimePrepareMaterialCaptureCache;

/// Comparison-only cache. Mesh SDF scene state remains the authoritative projected-object owner.
pub(super) struct RuntimePrepareMeshProjectionCache {
    initialized: bool,
    scene_meshes: Arc<[RenderMeshSnapshot]>,
    scene_mesh_world_bounds: Arc<[(u64, RenderMeshBounds)]>,
    clipmap_bounds: Vec<HybridGiGlobalSdfClipmapBounds>,
    material_capture: RuntimePrepareMaterialCaptureCache,
}

impl Default for RuntimePrepareMeshProjectionCache {
    fn default() -> Self {
        Self {
            initialized: false,
            scene_meshes: Arc::from([]),
            scene_mesh_world_bounds: Arc::from([]),
            clipmap_bounds: Vec::new(),
            material_capture: RuntimePrepareMaterialCaptureCache::default(),
        }
    }
}

impl RuntimePrepareMeshProjectionCache {
    pub(super) fn can_reuse(
        &self,
        scene_meshes: &[RenderMeshSnapshot],
        clipmap_bounds: &[HybridGiGlobalSdfClipmapBounds],
    ) -> bool {
        self.initialized
            && scene_meshes
                .iter()
                .map(|mesh| mesh.static_state)
                .all(RenderMeshStaticState::has_authoritative_revisions)
            && self.scene_meshes.as_ref() == scene_meshes
            && self.clipmap_bounds == clipmap_bounds
    }

    pub(super) fn capture(
        &mut self,
        scene_meshes: &[RenderMeshSnapshot],
        clipmap_bounds: &[HybridGiGlobalSdfClipmapBounds],
        scene_mesh_world_bounds: Arc<[(u64, RenderMeshBounds)]>,
    ) {
        let projection_complete = scene_mesh_world_bounds.len() == scene_meshes.len();
        self.scene_meshes = Arc::from(scene_meshes);
        self.scene_mesh_world_bounds = scene_mesh_world_bounds;
        self.clipmap_bounds.clear();
        self.clipmap_bounds.extend_from_slice(clipmap_bounds);
        self.initialized = projection_complete;
    }

    pub(super) fn refresh_material_capture(
        &mut self,
        context: &RuntimePrepareCollectorContext<'_>,
        scene_meshes: &[RenderMeshSnapshot],
    ) {
        self.material_capture =
            RuntimePrepareMaterialCaptureCache::from_context(context, scene_meshes);
    }

    pub(super) fn material_capture(&self) -> &RuntimePrepareMaterialCaptureCache {
        &self.material_capture
    }

    pub(super) fn scene_meshes(&self) -> Arc<[RenderMeshSnapshot]> {
        Arc::clone(&self.scene_meshes)
    }

    pub(super) fn scene_mesh_world_bounds(&self) -> Arc<[(u64, RenderMeshBounds)]> {
        Arc::clone(&self.scene_mesh_world_bounds)
    }
}

#[cfg(test)]
#[path = "tests/mesh_projection.rs"]
mod tests;
