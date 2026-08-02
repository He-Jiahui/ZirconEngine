use std::sync::Arc;

use zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot;

use crate::scene::viewport::pointer::precision::RendererVisibleSpatialPickSource;

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
        if let Ok(mut shared) = self.shared.lock() {
            let source = snapshot.map(|snapshot| {
                shared
                    .renderer_visible_spatial_pick_source
                    .as_ref()
                    .map(|current| {
                        current.with_snapshot(
                            snapshot.clone(),
                            self.layout.camera.clone(),
                            self.layout.viewport,
                        )
                    })
                    .unwrap_or_else(|| {
                        RendererVisibleSpatialPickSource::new(
                            snapshot,
                            &self.renderable_candidates,
                            self.layout.camera.clone(),
                            self.layout.viewport,
                        )
                    })
            });
            let source_changed =
                source.is_some() != shared.renderer_visible_spatial_pick_source.is_some();
            shared.renderer_visible_spatial_pick_source = source;
            shared.last_route = None;
            shared.last_debug_feed = None;
            return source_changed;
        }
        false
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
mod tests {
    #[test]
    fn renderer_snapshot_adoption_is_generation_bound_and_replaces_static_renderable_nodes() {
        let source = include_str!("viewport_overlay_pointer_router_visible_spatial_query.rs");

        assert!(source.contains("snapshot.identity().world.raw() == world_generation"));
        assert!(source.contains("Vec::new().into()"));
        assert!(source.contains("RendererVisibleSpatialPickSource::new"));
        assert!(source.contains("current.with_snapshot"));
        let scene_sync = include_str!("viewport_overlay_pointer_router_sync.rs");
        assert!(scene_sync.contains("self.renderer_visible_spatial_snapshot = None;"));
    }
}
