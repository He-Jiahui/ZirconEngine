use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

use crate::scene::viewport::{
    render_packet::build_scene_gizmos, HandleOverlayExtract, RenderMeshSnapshot,
    SceneGizmoOverlayExtract, SceneViewportSettings, ViewportCameraSnapshot,
};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

use super::{extract::ViewportInteractionExtract, key::ViewportInteractionExtractKey};

#[derive(Clone, Debug)]
struct CachedViewportInteractionExtract {
    key: ViewportInteractionExtractKey,
    extract: Arc<ViewportInteractionExtract>,
}

#[derive(Clone, Debug)]
pub(in crate::scene::viewport) enum ViewportInteractionExtractPointerResolution {
    Ready(Arc<ViewportInteractionExtract>),
    Stale,
    Preparing,
}

#[derive(Clone, Debug, Default)]
pub(in crate::scene::viewport) struct ViewportInteractionExtractCache {
    cached: RefCell<Option<CachedViewportInteractionExtract>>,
    pointer_rebuild_requested: Cell<bool>,
}

impl ViewportInteractionExtractCache {
    pub(in crate::scene::viewport) fn resolve_from_render_packet(
        &self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        render_meshes: &[RenderMeshSnapshot],
        build_handles: impl FnOnce() -> Vec<HandleOverlayExtract>,
        build_additional_gizmos: impl FnOnce() -> Vec<SceneGizmoOverlayExtract>,
    ) -> Arc<ViewportInteractionExtract> {
        let key = ViewportInteractionExtractKey::new(scene, selected, settings, camera, viewport);
        if let Some(extract) = self.cached_extract(&key) {
            self.pointer_rebuild_requested.set(false);
            return extract;
        }

        zircon_runtime::profile_counter!("editor", "interaction_extract_cache_miss", 1);
        let extract = self.rebuild(
            key,
            scene,
            selected,
            settings,
            camera,
            render_meshes,
            build_handles,
            build_additional_gizmos,
        );
        self.pointer_rebuild_requested.set(false);
        extract
    }

    pub(in crate::scene::viewport) fn resolve_for_pointer(
        &self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> ViewportInteractionExtractPointerResolution {
        let key = ViewportInteractionExtractKey::new(scene, selected, settings, camera, viewport);
        if let Some(extract) = self.cached_extract(&key) {
            self.pointer_rebuild_requested.set(false);
            return ViewportInteractionExtractPointerResolution::Ready(extract);
        }

        zircon_runtime::profile_counter!("editor", "interaction_extract_cache_miss", 1);
        if self.pointer_rebuild_requested.replace(true) {
            zircon_runtime::profile_counter!("editor", "interaction_extract_pointer_preparing", 1);
            ViewportInteractionExtractPointerResolution::Preparing
        } else {
            zircon_runtime::profile_counter!("editor", "interaction_extract_pointer_stale", 1);
            ViewportInteractionExtractPointerResolution::Stale
        }
    }

    pub(in crate::scene::viewport) fn invalidate(&self) {
        self.cached.replace(None);
        self.pointer_rebuild_requested.set(false);
    }

    fn cached_extract(
        &self,
        key: &ViewportInteractionExtractKey,
    ) -> Option<Arc<ViewportInteractionExtract>> {
        let extract = self
            .cached
            .borrow()
            .as_ref()
            .filter(|cached| cached.key == *key)
            .map(|cached| Arc::clone(&cached.extract));
        if extract.is_some() {
            zircon_runtime::profile_counter!("editor", "interaction_extract_cache_hit", 1);
        }
        extract
    }

    fn rebuild(
        &self,
        key: ViewportInteractionExtractKey,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        render_meshes: &[RenderMeshSnapshot],
        build_handles: impl FnOnce() -> Vec<HandleOverlayExtract>,
        build_additional_gizmos: impl FnOnce() -> Vec<SceneGizmoOverlayExtract>,
    ) -> Arc<ViewportInteractionExtract> {
        zircon_runtime::profile_scope!("editor", "viewport", "interaction_extract_rebuild");
        // This mirrors the owned payload copied by ViewportInteractionExtract::new. It is not an
        // allocator or process-memory measurement.
        zircon_runtime::profile_counter!(
            "editor",
            "interaction_mesh_copy_payload_bytes",
            render_mesh_snapshot_copy_payload_bytes(render_meshes)
        );
        let mut scene_gizmos = build_scene_gizmos(scene, selected, settings, camera);
        scene_gizmos.extend(build_additional_gizmos());
        let extract = Arc::new(ViewportInteractionExtract::new(
            build_handles(),
            scene_gizmos,
            render_meshes,
        ));
        self.cached.replace(Some(CachedViewportInteractionExtract {
            key,
            extract: Arc::clone(&extract),
        }));
        extract
    }
}

fn render_mesh_snapshot_copy_payload_bytes(render_meshes: &[RenderMeshSnapshot]) -> usize {
    render_meshes.iter().fold(
        render_meshes
            .len()
            .saturating_mul(std::mem::size_of::<RenderMeshSnapshot>()),
        |bytes, mesh| bytes.saturating_add(std::mem::size_of_val(mesh.morph_weights.as_slice())),
    )
}
