use super::super::{callback_dispatch, RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn asset_reference_pointer_event(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
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
        self.dispatch_asset_reference_pointer_press(surface_mode, list_kind, x, y, width, height);
    }

    pub(in crate::ui::retained_host::app) fn asset_reference_pointer_clicked(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) = self.prepare_asset_reference_pointer_target(
            surface_mode,
            list_kind,
            width,
            height,
            false,
        ) else {
            return;
        };
        if !self.sync_prepared_asset_reference_pointer_list(surface_mode, list_kind, &target, false)
        {
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
                self.write_asset_reference_pointer_state(
                    surface_mode,
                    list_kind,
                    dispatch.pointer.state,
                );
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_reference_pointer_moved(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) = self.prepare_asset_reference_pointer_target(
            surface_mode,
            list_kind,
            width,
            height,
            false,
        ) else {
            return;
        };
        let point = UiPoint::new(x, y);
        let Some(dispatch) = self.dispatch_prepared_asset_reference_pointer(
            surface_mode,
            list_kind,
            &target,
            false,
            |bridge| bridge.handle_move(point),
        ) else {
            return;
        };

        match dispatch {
            Ok(dispatch) => {
                self.write_asset_reference_pointer_state(surface_mode, list_kind, dispatch.state);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_reference_pointer_scrolled(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) = self.prepare_asset_reference_pointer_target(
            surface_mode,
            list_kind,
            width,
            height,
            false,
        ) else {
            return;
        };
        let point = UiPoint::new(x, y);
        let Some(dispatch) = self.dispatch_prepared_asset_reference_pointer(
            surface_mode,
            list_kind,
            &target,
            false,
            |bridge| bridge.handle_scroll(point, delta),
        ) else {
            return;
        };

        match dispatch {
            Ok(dispatch) => {
                self.write_asset_reference_pointer_state(surface_mode, list_kind, dispatch.state);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
