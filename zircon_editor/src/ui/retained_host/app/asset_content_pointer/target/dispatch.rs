use super::super::super::RetainedEditorHost;
use super::PreparedAssetContentPointerTarget;
use crate::ui::retained_host::asset_pointer::AssetContentListPointerBridge;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_prepared_asset_content_pointer_list(
        &mut self,
        surface_mode: &str,
        target: &PreparedAssetContentPointerTarget,
        clear_drag_on_error: bool,
    ) -> bool {
        self.patch_prepared_asset_content_pointer_geometry(
            surface_mode,
            target,
            clear_drag_on_error,
        )
    }

    pub(in crate::ui::retained_host::app) fn dispatch_prepared_asset_content_pointer<T>(
        &mut self,
        surface_mode: &str,
        target: &PreparedAssetContentPointerTarget,
        clear_drag_on_error: bool,
        dispatch: impl FnOnce(&mut AssetContentListPointerBridge) -> T,
    ) -> Option<T> {
        if !self.patch_prepared_asset_content_pointer_geometry(
            surface_mode,
            target,
            clear_drag_on_error,
        ) {
            return None;
        }
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        Some(dispatch(&mut surface.content_bridge))
    }

    fn patch_prepared_asset_content_pointer_geometry(
        &mut self,
        surface_mode: &str,
        target: &PreparedAssetContentPointerTarget,
        clear_drag_on_error: bool,
    ) -> bool {
        let state_changed = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.clear_asset_content_drag_on_error(clear_drag_on_error);
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return false;
            };
            if surface.content_size == target.content_size {
                return true;
            }
            surface.content_size = target.content_size;
            if let Some(state) = surface.content_bridge.sync_pane_size(surface.content_size) {
                surface.content_state = state;
                true
            } else {
                false
            }
        };
        if state_changed {
            self.apply_asset_pointer_state_to_ui(surface_mode);
        }
        true
    }
}
