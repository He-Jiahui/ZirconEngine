use super::super::super::{RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
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
        let Some(state) = self.dispatch_prepared_asset_reference_pointer(
            surface_mode,
            list_kind,
            &target,
            false,
            |bridge| bridge.update_hovered_row(point),
        ) else {
            return;
        };

        if let Some(state) = state {
            self.write_asset_reference_pointer_state(surface_mode, list_kind, state);
        } else {
            self.clear_inactive_asset_reference_pointer_hover(surface_mode, list_kind);
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
