use super::super::super::RetainedEditorHost;
use super::PreparedAssetReferencePointerTarget;
use crate::ui::retained_host::asset_pointer::AssetReferenceListPointerBridge;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_prepared_asset_reference_pointer_list(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        target: &PreparedAssetReferencePointerTarget,
        clear_drag_on_error: bool,
    ) -> bool {
        self.patch_prepared_asset_reference_pointer_geometry(
            surface_mode,
            list_kind,
            target,
            clear_drag_on_error,
        )
    }

    pub(in crate::ui::retained_host::app) fn dispatch_prepared_asset_reference_pointer<T>(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        target: &PreparedAssetReferencePointerTarget,
        clear_drag_on_error: bool,
        dispatch: impl FnOnce(&mut AssetReferenceListPointerBridge) -> T,
    ) -> Option<T> {
        if !self.patch_prepared_asset_reference_pointer_geometry(
            surface_mode,
            list_kind,
            target,
            clear_drag_on_error,
        ) {
            return None;
        }
        let surface = self.asset_surface_pointer_state_mut(surface_mode)?;
        let list = surface.reference_list_mut(list_kind)?;
        Some(dispatch(&mut list.bridge))
    }

    fn patch_prepared_asset_reference_pointer_geometry(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        target: &PreparedAssetReferencePointerTarget,
        clear_drag_on_error: bool,
    ) -> bool {
        let state_changed = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.clear_asset_reference_drag_on_error(clear_drag_on_error);
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return false;
            };
            let Some(list) = surface.reference_list_mut(list_kind) else {
                self.clear_asset_reference_drag_on_error(clear_drag_on_error);
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return false;
            };
            if list.size == target.list_size {
                return true;
            }
            list.size = target.list_size;
            if let Some(state) = list.bridge.sync_pane_size(list.size) {
                list.state = state;
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
