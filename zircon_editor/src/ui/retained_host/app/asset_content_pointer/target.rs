use super::super::*;
use crate::ui::retained_host::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerDispatch, AssetContentListPointerLayout,
    AssetListPointerState,
};
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(in crate::ui::retained_host::app) struct PreparedAssetContentPointerTarget {
    pub snapshot: AssetWorkspaceSnapshot,
    content_size: UiSize,
}

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

    pub(in crate::ui::retained_host::app) fn sync_prepared_asset_content_pointer_list(
        &mut self,
        surface_mode: &str,
        target: &PreparedAssetContentPointerTarget,
        clear_drag_on_error: bool,
    ) -> bool {
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return false;
        };
        surface.content_size = target.content_size;
        surface.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(&target.snapshot, surface.content_size),
            surface.content_state.clone(),
        );
        true
    }

    pub(in crate::ui::retained_host::app) fn dispatch_prepared_asset_content_pointer(
        &mut self,
        surface_mode: &str,
        target: &PreparedAssetContentPointerTarget,
        clear_drag_on_error: bool,
        dispatch: impl FnOnce(
            &mut AssetContentListPointerBridge,
        ) -> Result<AssetContentListPointerDispatch, String>,
    ) -> Option<Result<AssetContentListPointerDispatch, String>> {
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        surface.content_size = target.content_size;
        surface.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(&target.snapshot, surface.content_size),
            surface.content_state.clone(),
        );
        Some(dispatch(&mut surface.content_bridge))
    }

    pub(in crate::ui::retained_host::app) fn write_asset_content_pointer_state(
        &mut self,
        surface_mode: &str,
        state: AssetListPointerState,
    ) {
        if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
            surface.content_state = state;
        }
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }

    fn clear_asset_content_drag_on_error(&mut self, clear_drag_on_error: bool) {
        if clear_drag_on_error {
            self.active_asset_drag_payload = None;
        }
    }
}
