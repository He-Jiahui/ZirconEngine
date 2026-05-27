use zircon_runtime::core::framework::picking::PickingDebugFeed;
use zircon_runtime_interface::ui::layout::UiPoint;

use crate::scene::viewport::pointer::runtime_picking_adapter::runtime_debug_feed_for_candidates;

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn debug_feed_at(&self, point: UiPoint) -> Result<PickingDebugFeed, String> {
        let hit = self.surface.hit_test(point);
        let shared = self
            .shared
            .lock()
            .map_err(|_| "viewport pointer shared resolution lock poisoned".to_string())?;
        Ok(runtime_debug_feed_for_candidates(
            &shared.candidates,
            &hit.stacked,
            point,
        ))
    }
}
