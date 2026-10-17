//! 布局与场景同步决定候选重建，提取身份改变时撤销旧渲染快照；外部渲染提交后才能采用新空间查询来源。

use std::sync::Arc;

use crate::scene::viewport::{ViewportCameraSnapshot, ViewportInteractionExtract};
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::pointer::{
    candidates::renderable_candidates, viewport_pointer_layout::ViewportPointerLayout,
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn clear_scene(&mut self) -> bool {
        // BUG: [CR-EDITOR-SP-0005] 带副作用的清理串在短路条件里；提取存在时后续代际和渲染快照不会被取走，清场后仍保留旧空间来源。
        let had_extract = self.interaction_extract.take().is_some();
        let had_generation = self.scene_world_generation.take().is_some();
        let had_snapshot = self.renderer_visible_spatial_snapshot.take().is_some();
        let had_scene =
            had_extract || had_generation || had_snapshot || !self.renderable_candidates.is_empty();
        self.renderable_candidates = Vec::new().into();
        let layout_changed = self.sync(ViewportPointerLayout::default());
        let source_changed = self.refresh_renderer_visible_spatial_pick_source();
        had_scene || layout_changed || source_changed
    }

    pub(crate) fn sync(&mut self, layout: ViewportPointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        self.interaction_extract = None;
        self.rebuild_surface();
        true
    }

    /// 仅接收渲染已发布且与相机、尺寸一致的提取；采用它时旧渲染空间来源失效，等待新的提交快照。
    pub(crate) fn sync_scene(
        &mut self,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        world_generation: u64,
        interaction_extract: Arc<ViewportInteractionExtract>,
    ) -> bool {
        if self
            .interaction_extract
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &interaction_extract))
            && self.scene_world_generation == Some(world_generation)
        {
            return false;
        }

        // The extract key includes the camera, viewport extent, settings and world generation.
        // A query snapshot is valid only for the exact rendered extract; fail closed until the
        // next successful submission publishes a replacement.
        self.renderer_visible_spatial_snapshot = None;
        self.scene_world_generation = Some(world_generation);
        self.renderable_candidates =
            renderable_candidates(interaction_extract.render_meshes()).into();

        let changed = self.sync(ViewportPointerLayout {
            viewport,
            camera: camera.clone(),
            handles: interaction_extract.handles(),
            scene_gizmos: interaction_extract.scene_gizmos(),
            renderables: self.layout_renderables(),
        });
        self.interaction_extract = Some(interaction_extract);
        self.refresh_renderer_visible_spatial_pick_source();
        changed
    }
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    use crate::scene::viewport::{
        render_packet::build_render_packet, SceneViewportSettings, ViewportInteractionExtractCache,
    };
    use zircon_runtime::scene::Scene;
    #[test]
    fn clearing_extract_also_retires_its_world_generation() {
        let scene = Scene::new();
        let settings = SceneViewportSettings::default();
        let camera = ViewportCameraSnapshot::default();
        let viewport = UVec2::new(320, 240);
        let cache = ViewportInteractionExtractCache::default();
        let packet = build_render_packet(&scene, &settings, &camera, None, viewport);
        let extract = cache.resolve_from_render_packet(
            &scene,
            None,
            &settings,
            &camera,
            viewport,
            &packet.scene.meshes,
            Vec::new,
            Vec::new,
        );
        let mut router = ViewportOverlayPointerRouter::new();
        router.sync_scene(&camera, viewport, scene.world_generation(), extract);
        assert!(router.interaction_extract.is_some());
        assert!(router.scene_world_generation.is_some());
        assert!(router.clear_scene());
        assert!(router.interaction_extract.is_none());
        assert_eq!(router.scene_world_generation, None);
        assert!(router.renderer_visible_spatial_snapshot.is_none());
        assert!(router.renderable_candidates.is_empty());
        assert!(!router.clear_scene(), "already cleared state is idempotent");
    }
}
