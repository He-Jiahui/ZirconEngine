use super::super::super::{callback_dispatch, RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
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
}
