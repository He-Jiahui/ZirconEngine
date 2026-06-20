use super::super::super::RetainedEditorHost;
use super::PreparedAssetReferencePointerTarget;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn prepare_asset_reference_pointer_target(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        width: f32,
        height: f32,
        clear_drag_on_error: bool,
    ) -> Option<PreparedAssetReferencePointerTarget> {
        let Some(snapshot) = self.asset_workspace_snapshot_for_pointer(surface_mode) else {
            self.clear_asset_reference_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        let Some(list_size) = self
            .asset_surface_pointer_state(surface_mode)
            .and_then(|surface| surface.reference_list(list_kind))
            .and_then(|list| {
                self.resolve_callback_surface_size_for_asset_surface(
                    surface_mode,
                    width,
                    height,
                    list.size,
                )
            })
        else {
            self.clear_asset_reference_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!(
                "Unknown asset reference pointer target {surface_mode}/{list_kind}"
            ));
            return None;
        };

        Some(PreparedAssetReferencePointerTarget {
            snapshot,
            list_size,
        })
    }

    pub(in crate::ui::retained_host::app::asset_reference_pointer::target) fn clear_asset_reference_drag_on_error(
        &mut self,
        clear_drag_on_error: bool,
    ) {
        if clear_drag_on_error {
            self.active_asset_drag_payload = None;
        }
    }
}
