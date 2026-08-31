use std::sync::Arc;

use super::ViewportOverlayPointerRouter;

impl Clone for ViewportOverlayPointerRouter {
    fn clone(&self) -> Self {
        let mut clone = Self::new();
        clone.layout = self.layout.clone();
        clone.interaction_extract = self.interaction_extract.as_ref().map(Arc::clone);
        clone.renderable_candidates = Arc::clone(&self.renderable_candidates);
        clone.scene_world_generation = self.scene_world_generation;
        clone.renderer_visible_spatial_snapshot = self.renderer_visible_spatial_snapshot.clone();
        clone.retained_candidate_ids = Arc::clone(&self.retained_candidate_ids);
        clone.refresh_renderer_visible_spatial_pick_source();
        clone.rebuild_surface();
        clone
    }
}
