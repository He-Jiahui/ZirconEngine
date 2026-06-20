use super::super::super::RetainedEditorHost;
use super::PreparedAssetContentPointerTarget;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn prepare_asset_content_pointer_target(
        &mut self,
        surface_mode: &str,
        width: f32,
        height: f32,
        clear_drag_on_error: bool,
    ) -> Option<PreparedAssetContentPointerTarget> {
        let Some(snapshot) = self.asset_workspace_snapshot_for_pointer(surface_mode) else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        let Some(content_size) =
            self.asset_surface_pointer_state(surface_mode)
                .and_then(|surface| {
                    self.resolve_callback_surface_size_for_asset_surface(
                        surface_mode,
                        width,
                        height,
                        surface.content_size,
                    )
                })
        else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };

        Some(PreparedAssetContentPointerTarget {
            snapshot,
            content_size,
        })
    }

    pub(in crate::ui::retained_host::app::asset_content_pointer::target) fn clear_asset_content_drag_on_error(
        &mut self,
        clear_drag_on_error: bool,
    ) {
        if clear_drag_on_error {
            self.active_asset_drag_payload = None;
        }
    }
}
