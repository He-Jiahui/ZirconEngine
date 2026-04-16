use super::*;

impl SlintEditorHost {
    pub(super) fn asset_content_pointer_clicked(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.recompute_if_dirty();
        self.focus_callback_source_window();
        let Some(snapshot) = self.asset_workspace_snapshot_for_pointer(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };

        {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            };
            surface.content_size = UiSize::new(width.max(0.0), height.max(0.0));
            surface.content_bridge.sync(
                AssetContentListPointerLayout::from_snapshot(&snapshot, surface.content_size),
                surface.content_state.clone(),
            );
        }

        let runtime = &self.runtime;
        let bridge = &self.asset_surface_bridge;
        let point = UiPoint::new(x, y);
        let dispatch = match surface_mode {
            "activity" => callback_dispatch::dispatch_shared_asset_content_pointer_click(
                runtime,
                bridge,
                &mut self.activity_asset_pointer.content_bridge,
                point,
            ),
            "browser" => callback_dispatch::dispatch_shared_asset_content_pointer_click(
                runtime,
                bridge,
                &mut self.browser_asset_pointer.content_bridge,
                point,
            ),
            _ => {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            }
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    surface.content_state = dispatch.pointer.state;
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn asset_content_pointer_moved(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.recompute_if_dirty();
        self.focus_callback_source_window();
        let Some(snapshot) = self.asset_workspace_snapshot_for_pointer(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };

        let dispatch = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            };
            surface.content_size = UiSize::new(width.max(0.0), height.max(0.0));
            surface.content_bridge.sync(
                AssetContentListPointerLayout::from_snapshot(&snapshot, surface.content_size),
                surface.content_state.clone(),
            );
            surface.content_bridge.handle_move(UiPoint::new(x, y))
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    surface.content_state = dispatch.state;
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn asset_content_pointer_scrolled(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.recompute_if_dirty();
        self.focus_callback_source_window();
        let Some(snapshot) = self.asset_workspace_snapshot_for_pointer(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };

        let dispatch = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            };
            surface.content_size = UiSize::new(width.max(0.0), height.max(0.0));
            surface.content_bridge.sync(
                AssetContentListPointerLayout::from_snapshot(&snapshot, surface.content_size),
                surface.content_state.clone(),
            );
            surface
                .content_bridge
                .handle_scroll(UiPoint::new(x, y), delta)
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    surface.content_state = dispatch.state;
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
