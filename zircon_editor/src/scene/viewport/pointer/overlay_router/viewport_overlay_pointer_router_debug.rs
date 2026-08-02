use zircon_runtime::core::framework::picking::PickingDebugFeed;
use zircon_runtime_interface::ui::layout::UiPoint;

use crate::scene::viewport::pointer::runtime_picking_adapter::resolve_runtime_route_and_debug_feed_with_renderer_candidates;

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn debug_feed_at(&self, point: UiPoint) -> Result<PickingDebugFeed, String> {
        let hit = self.surface.hit_test(point);
        let shared = self
            .shared
            .lock()
            .map_err(|_| "viewport pointer shared resolution lock poisoned".to_string())?;
        let renderer_candidates = shared
            .renderer_visible_spatial_pick_source
            .as_ref()
            .map(|source| source.candidates_at(point))
            .unwrap_or_default();
        let (_, debug_feed) = resolve_runtime_route_and_debug_feed_with_renderer_candidates(
            &shared.candidates,
            &hit.stacked,
            point,
            &renderer_candidates,
        );
        Ok(debug_feed)
    }
}
