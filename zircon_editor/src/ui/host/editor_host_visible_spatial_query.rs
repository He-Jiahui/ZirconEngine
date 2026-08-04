use zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot;

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn sync_renderer_visible_spatial_snapshot(
        &self,
        snapshot: Option<RenderVisibleSpatialQuerySnapshot>,
    ) {
        self.shell()
            .lock()
            .state
            .sync_renderer_visible_spatial_snapshot(snapshot);
    }
}
