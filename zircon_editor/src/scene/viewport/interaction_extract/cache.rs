use std::{cell::RefCell, sync::Arc};

use crate::scene::viewport::{
    render_packet::{build_render_packet, build_scene_gizmos},
    HandleOverlayExtract, RenderMeshSnapshot, SceneGizmoOverlayExtract, SceneViewportSettings,
    ViewportCameraSnapshot,
};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

use super::{extract::ViewportInteractionExtract, key::ViewportInteractionExtractKey};

#[derive(Clone, Debug)]
struct CachedViewportInteractionExtract {
    key: ViewportInteractionExtractKey,
    extract: Arc<ViewportInteractionExtract>,
}

#[derive(Clone, Debug, Default)]
pub(in crate::scene::viewport) struct ViewportInteractionExtractCache {
    cached: RefCell<Option<CachedViewportInteractionExtract>>,
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
            return extract;
        }

        zircon_runtime::profile_counter!("editor", "interaction_extract_cache_miss", 1);
        self.rebuild(
            key,
            scene,
            selected,
            settings,
            camera,
            render_meshes,
            build_handles,
            build_additional_gizmos,
        )
    }

    pub(in crate::scene::viewport) fn resolve_for_pointer(
        &self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        build_handles: impl FnOnce() -> Vec<HandleOverlayExtract>,
        build_additional_gizmos: impl FnOnce() -> Vec<SceneGizmoOverlayExtract>,
    ) -> Arc<ViewportInteractionExtract> {
        let key = ViewportInteractionExtractKey::new(scene, selected, settings, camera, viewport);
        if let Some(extract) = self.cached_extract(&key) {
            return extract;
        }

        zircon_runtime::profile_counter!("editor", "interaction_extract_cache_miss", 1);
        let packet = {
            zircon_runtime::profile_scope!("editor", "viewport", "pointer_fallback_packet_build");
            build_render_packet(scene, settings, camera, selected, viewport)
        };
        self.rebuild(
            key,
            scene,
            selected,
            settings,
            camera,
            &packet.scene.meshes,
            build_handles,
            build_additional_gizmos,
        )
    }

    pub(in crate::scene::viewport) fn invalidate(&self) {
        self.cached.replace(None);
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
