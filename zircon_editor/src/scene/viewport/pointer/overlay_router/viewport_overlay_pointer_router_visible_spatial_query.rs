//! 采用同世界代际的渲染空间快照后，静态渲染物候选退出表面；相机或尺寸变化重建投影，事件只投影空间查询命中的 owner。

use std::sync::Arc;

use zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot;

use crate::scene::viewport::pointer::precision::{
    lock_shared_resolution_state, RendererVisibleSpatialPickSource,
};

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn sync_renderer_visible_spatial_snapshot(
        &mut self,
        world_generation: u64,
        snapshot: Option<RenderVisibleSpatialQuerySnapshot>,
    ) -> bool {
        let snapshot =
            snapshot.filter(|snapshot| snapshot.identity().world.raw() == world_generation);
        let changed = self
            .renderer_visible_spatial_snapshot
            .as_ref()
            .map(|current| current.identity())
            != snapshot.as_ref().map(|next| next.identity());
        self.scene_world_generation = Some(world_generation);
        self.renderer_visible_spatial_snapshot = snapshot;
        let source_changed = self.refresh_renderer_visible_spatial_pick_source();
        let renderables = self.layout_renderables();
        if self.layout.renderables != renderables {
            self.layout.renderables = renderables;
            self.rebuild_surface();
        }
        changed || source_changed
    }

    pub(super) fn layout_renderables(
        &self,
    ) -> Arc<[crate::scene::viewport::pointer::viewport_renderable_pick_candidate::ViewportRenderablePickCandidate]>{
        if self.has_renderer_visible_spatial_snapshot() {
            Vec::new().into()
        } else {
            Arc::clone(&self.renderable_candidates)
        }
    }

    pub(super) fn refresh_renderer_visible_spatial_pick_source(&mut self) -> bool {
        let snapshot = self
            .renderer_visible_spatial_snapshot
            .as_ref()
            .filter(|snapshot| {
                self.scene_world_generation
                    .is_some_and(|world| snapshot.identity().world.raw() == world)
            })
            .cloned();
        let mut shared = lock_shared_resolution_state(self.shared.as_ref());
        let (source, source_changed, source_reused) = match snapshot {
            Some(snapshot) => match shared.renderer_visible_spatial_pick_source.as_ref() {
                Some(current)
                    if current.is_current_for(
                        &snapshot,
                        &self.layout.camera,
                        self.layout.viewport,
                    ) =>
                {
                    (Some(current.clone()), false, true)
                }
                Some(current) => (
                    Some(current.with_snapshot(
                        snapshot,
                        self.layout.camera.clone(),
                        self.layout.viewport,
                    )),
                    true,
                    false,
                ),
                None => (
                    Some(RendererVisibleSpatialPickSource::new(
                        snapshot,
                        &self.renderable_candidates,
                        self.layout.camera.clone(),
                        self.layout.viewport,
                    )),
                    true,
                    false,
                ),
            },
            None => (
                None,
                shared.renderer_visible_spatial_pick_source.is_some(),
                false,
            ),
        };
        let projection_context_build_count = if source_changed && source.is_some() {
            1_usize
        } else {
            0
        };
        let source_reuse_count = if source_reused { 1_usize } else { 0 };
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_projection_context_build_count",
            projection_context_build_count,
        );
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_source_reuse_count",
            source_reuse_count,
        );
        shared.renderer_visible_spatial_pick_source = source;
        if source_changed {
            shared.last_route = None;
            shared.last_debug_feed = None;
        }
        source_changed
    }

    fn has_renderer_visible_spatial_snapshot(&self) -> bool {
        self.renderer_visible_spatial_snapshot
            .as_ref()
            .is_some_and(|snapshot| {
                self.scene_world_generation
                    .is_some_and(|world| snapshot.identity().world.raw() == world)
            })
    }
}

#[cfg(test)]
#[path = "tests/viewport_overlay_pointer_router_visible_spatial_query.rs"]
mod tests;
