use super::super::super::RetainedEditorHost;
use super::PreparedAssetContentPointerTarget;
use crate::ui::retained_host::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerDispatch, AssetContentListPointerLayout,
};
use crate::ui::workbench::asset_content_layout::AssetContentSurfaceProfile;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_prepared_asset_content_pointer_list(
        &mut self,
        surface_mode: &str,
        target: &PreparedAssetContentPointerTarget,
        clear_drag_on_error: bool,
    ) -> bool {
        let Some(surface_profile) = AssetContentSurfaceProfile::from_surface_mode(surface_mode)
        else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return false;
        };
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return false;
        };
        if surface.content_size != target.content_size {
            surface.content_size = target.content_size;
            surface.content_bridge.sync(
                AssetContentListPointerLayout::from_snapshot(
                    target.snapshot.as_ref(),
                    surface.content_size,
                    surface_profile,
                ),
                surface.content_state.clone(),
            );
        }
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
        let Some(surface_profile) = AssetContentSurfaceProfile::from_surface_mode(surface_mode)
        else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.clear_asset_content_drag_on_error(clear_drag_on_error);
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return None;
        };
        if surface.content_size != target.content_size {
            surface.content_size = target.content_size;
            surface.content_bridge.sync(
                AssetContentListPointerLayout::from_snapshot(
                    target.snapshot.as_ref(),
                    surface.content_size,
                    surface_profile,
                ),
                surface.content_state.clone(),
            );
        }
        Some(dispatch(&mut surface.content_bridge))
    }
}
