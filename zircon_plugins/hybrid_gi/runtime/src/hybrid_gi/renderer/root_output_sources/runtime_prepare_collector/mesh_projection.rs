use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState,
};
use zircon_runtime::graphics::RuntimePrepareCollectorContext;

use crate::hybrid_gi::scene_representation::HybridGiGlobalSdfClipmapBounds;

use super::material_capture::RuntimePrepareMaterialCaptureCache;

/// Comparison-only cache. Mesh SDF scene state remains the authoritative projected-object owner.
pub(super) struct RuntimePrepareMeshSdfProjectionCache {
    initialized: bool,
    scene_meshes: Arc<[RenderMeshSnapshot]>,
    scene_mesh_world_bounds: Arc<[(u64, RenderMeshBounds)]>,
    clipmap_bounds: Vec<HybridGiGlobalSdfClipmapBounds>,
    material_capture: RuntimePrepareMaterialCaptureCache,
}

impl Default for RuntimePrepareMeshSdfProjectionCache {
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

impl RuntimePrepareMeshSdfProjectionCache {
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
        self.scene_meshes = Arc::from(scene_meshes);
        self.scene_mesh_world_bounds = scene_mesh_world_bounds;
        self.clipmap_bounds.clear();
        self.clipmap_bounds.extend_from_slice(clipmap_bounds);
        self.initialized = true;
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
mod tests {
    use std::sync::Arc;

    use zircon_runtime::core::framework::render::{
        RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
    };
    use zircon_runtime::core::framework::scene::Mobility;
    use zircon_runtime::core::math::{Transform, Vec3, Vec4};
    use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    use super::RuntimePrepareMeshSdfProjectionCache;
    use crate::hybrid_gi::scene_representation::HybridGiGlobalSdfClipmapBounds;

    #[test]
    fn cache_requires_authoritative_static_revisions() {
        assert!(RenderMeshStaticState::new(true, 7, 11).has_authoritative_revisions());
        assert!(!RenderMeshStaticState::new(false, 7, 11).has_authoritative_revisions());
        assert!(!RenderMeshStaticState::new(true, 0, 11).has_authoritative_revisions());
        assert!(!RenderMeshStaticState::new(true, 7, 0).has_authoritative_revisions());
    }

    #[test]
    fn cache_reuses_only_the_same_page_aligned_clipmap_snapshot() {
        let initial = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0)];
        let shifted = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::X * 4.0, 16.0)];
        let mut cache = RuntimePrepareMeshSdfProjectionCache::default();

        assert!(!cache.can_reuse(&[], &initial));
        cache.capture(&[], &initial, Arc::from([]));
        assert!(cache.can_reuse(&[], &initial));
        assert!(!cache.can_reuse(&[], &shifted));
    }

    #[test]
    fn cache_never_reuses_a_dynamic_mesh_snapshot() {
        let clipmaps = [HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0)];
        let meshes = [RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1,
            transform_revision: 1,
            transform: Transform::from_translation(Vec3::ZERO),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/cache-test.model.toml",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/cache-test.zmaterial",
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: RenderMeshStaticState::new(false, 7, 11),
            common: RendererCommon::default(),
        }];
        let mut cache = RuntimePrepareMeshSdfProjectionCache::default();

        cache.capture(&meshes, &clipmaps, Arc::from([]));

        assert!(!cache.can_reuse(&meshes, &clipmaps));
    }
}
