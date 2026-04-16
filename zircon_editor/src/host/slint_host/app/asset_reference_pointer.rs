use super::*;

impl SlintEditorHost {
    pub(super) fn asset_reference_pointer_clicked(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
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
            let Some(list) = surface.reference_list_mut(list_kind) else {
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return;
            };
            list.size = UiSize::new(width.max(0.0), height.max(0.0));
            let Some(layout) = Self::asset_reference_layout(&snapshot, list_kind, list.size) else {
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return;
            };
            list.bridge.sync(layout, list.state.clone());
        }

        let runtime = &self.runtime;
        let bridge = &self.asset_surface_bridge;
        let point = UiPoint::new(x, y);
        let dispatch = match (surface_mode, list_kind) {
            ("activity", "references") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.activity_asset_pointer.references.bridge,
                    point,
                )
            }
            ("activity", "used_by") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.activity_asset_pointer.used_by.bridge,
                    point,
                )
            }
            ("browser", "references") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.browser_asset_pointer.references.bridge,
                    point,
                )
            }
            ("browser", "used_by") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.browser_asset_pointer.used_by.bridge,
                    point,
                )
            }
            _ => {
                self.set_status_line(format!(
                    "Unknown asset reference pointer target {surface_mode}/{list_kind}"
                ));
                return;
            }
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    if let Some(list) = surface.reference_list_mut(list_kind) {
                        list.state = dispatch.pointer.state;
                    }
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn asset_reference_pointer_moved(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
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
            let Some(list) = surface.reference_list_mut(list_kind) else {
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return;
            };
            list.size = UiSize::new(width.max(0.0), height.max(0.0));
            let Some(layout) = Self::asset_reference_layout(&snapshot, list_kind, list.size) else {
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return;
            };
            list.bridge.sync(layout, list.state.clone());
            list.bridge.handle_move(UiPoint::new(x, y))
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    if let Some(list) = surface.reference_list_mut(list_kind) {
                        list.state = dispatch.state;
                    }
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn asset_reference_pointer_scrolled(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
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
            let Some(list) = surface.reference_list_mut(list_kind) else {
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return;
            };
            list.size = UiSize::new(width.max(0.0), height.max(0.0));
            let Some(layout) = Self::asset_reference_layout(&snapshot, list_kind, list.size) else {
                self.set_status_line(format!("Unknown asset reference list {list_kind}"));
                return;
            };
            list.bridge.sync(layout, list.state.clone());
            list.bridge.handle_scroll(UiPoint::new(x, y), delta)
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    if let Some(list) = surface.reference_list_mut(list_kind) {
                        list.state = dispatch.state;
                    }
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
