use super::super::{callback_dispatch, RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn asset_content_pointer_event(
        &mut self,
        surface_mode: &str,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if button == 1 && kind == 2 {
            self.active_asset_drag_payload = None;
            return;
        }
        if kind != 0 || button != 1 {
            return;
        }
        self.dispatch_asset_content_pointer_press(surface_mode, x, y, width, height);
    }

    pub(in crate::ui::retained_host::app) fn asset_content_pointer_clicked(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            return;
        };
        if !self.sync_prepared_asset_content_pointer_list(surface_mode, &target, false) {
            return;
        }

        if !self.ensure_asset_surface_bridge() {
            return;
        }
        let Some(bridge) = self.asset_surface_bridge.as_ref() else {
            self.set_status_line("Asset UI controls are not available");
            return;
        };
        let runtime = &self.runtime;
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
                self.write_asset_content_pointer_state(surface_mode, dispatch.pointer.state);
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_content_pointer_moved(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            return;
        };
        let point = UiPoint::new(x, y);
        let Some(dispatch) =
            self.dispatch_prepared_asset_content_pointer(surface_mode, &target, false, |bridge| {
                bridge.handle_move(point)
            })
        else {
            return;
        };

        match dispatch {
            Ok(dispatch) => {
                self.write_asset_content_pointer_state(surface_mode, dispatch.state);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_content_pointer_scrolled(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            return;
        };
        let point = UiPoint::new(x, y);
        let Some(dispatch) =
            self.dispatch_prepared_asset_content_pointer(surface_mode, &target, false, |bridge| {
                bridge.handle_scroll(point, delta)
            })
        else {
            return;
        };

        match dispatch {
            Ok(dispatch) => {
                self.write_asset_content_pointer_state(surface_mode, dispatch.state);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
