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
        let had_scene = self.interaction_extract.take().is_some()
            || self.scene_world_generation.take().is_some()
            || self.renderer_visible_spatial_snapshot.take().is_some()
            || !self.renderable_candidates.is_empty();
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
