use super::super::*;
use crate::ui::retained_host::asset_pointer::{
    AssetListPointerState, AssetReferenceListPointerBridge, AssetReferenceListPointerDispatch,
};
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(in crate::ui::retained_host::app) struct PreparedAssetReferencePointerTarget {
    pub snapshot: AssetWorkspaceSnapshot,
    list_size: UiSize,
}

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
        list.size = target.list_size;
        let Some(layout) =
            Self::asset_reference_layout(&target.snapshot, list_kind, target.list_size)
        else {
            self.clear_asset_reference_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset reference list {list_kind}"));
            return false;
        };
        list.bridge.sync(layout, list.state.clone());
        true
    }

    pub(in crate::ui::retained_host::app) fn dispatch_prepared_asset_reference_pointer(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        target: &PreparedAssetReferencePointerTarget,
        clear_drag_on_error: bool,
        dispatch: impl FnOnce(
            &mut AssetReferenceListPointerBridge,
        ) -> Result<AssetReferenceListPointerDispatch, String>,
    ) -> Option<Result<AssetReferenceListPointerDispatch, String>> {
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
        list.size = target.list_size;
        let Some(layout) =
            Self::asset_reference_layout(&target.snapshot, list_kind, target.list_size)
        else {
            self.clear_asset_reference_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset reference list {list_kind}"));
            return None;
        };
        list.bridge.sync(layout, list.state.clone());
        Some(dispatch(&mut list.bridge))
    }

    pub(in crate::ui::retained_host::app) fn write_asset_reference_pointer_state(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        state: AssetListPointerState,
    ) {
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            if let Some(list) = surface.reference_list_mut(list_kind) {
                list.state = state;
            }
        }
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }

    fn clear_asset_reference_drag_on_error(&mut self, clear_drag_on_error: bool) {
        if clear_drag_on_error {
            self.active_asset_drag_payload = None;
        }
    }
}
