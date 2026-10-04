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

    pub(crate) fn sync_renderer_visible_spatial_snapshot_for_view(
        &self,
        view_id: &crate::core::editor_event::ViewInstanceId,
        snapshot: Option<RenderVisibleSpatialQuerySnapshot>,
    ) {
        self.shell()
            .lock()
            .state
            .sync_renderer_visible_spatial_snapshot_for_view(view_id, snapshot);
    }
}
