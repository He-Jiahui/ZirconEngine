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
        if list.size != target.list_size {
            list.size = target.list_size;
            let Some(layout) =
                Self::asset_reference_layout(target.snapshot.as_ref(), list_kind, target.list_size)
            else {
                self.clear_asset_reference_drag_on_error(clear_drag_on_error);
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return false;
            };
            list.bridge.sync(layout, list.state.clone());
        }
        true
    }

    pub(in crate::ui::retained_host::app) fn dispatch_prepared_asset_reference_pointer<T>(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        target: &PreparedAssetReferencePointerTarget,
        clear_drag_on_error: bool,
        dispatch: impl FnOnce(&mut AssetReferenceListPointerBridge) -> T,
    ) -> Option<T> {
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.clear_asset_reference_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        let Some(list) = surface.reference_list_mut(list_kind) else {
            self.clear_asset_reference_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset reference list {list_kind}"));
            return None;
        };
        if list.size != target.list_size {
            list.size = target.list_size;
            let Some(layout) =
                Self::asset_reference_layout(target.snapshot.as_ref(), list_kind, target.list_size)
            else {
                self.clear_asset_reference_drag_on_error(clear_drag_on_error);
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return None;
            };
            list.bridge.sync(layout, list.state.clone());
        }
        Some(dispatch(&mut list.bridge))
    }
}
