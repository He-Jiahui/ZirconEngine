use super::super::RetainedEditorHost;

mod click;
mod motion;

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
}
