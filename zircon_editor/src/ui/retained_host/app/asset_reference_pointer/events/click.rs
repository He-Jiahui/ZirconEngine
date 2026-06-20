use super::super::super::{RetainedEditorHost, UiPoint};

mod dispatch;

impl RetainedEditorHost {
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

        self.dispatch_asset_reference_pointer_click_to_bridge(
            surface_mode,
            list_kind,
            UiPoint::new(x, y),
        );
    }
}
