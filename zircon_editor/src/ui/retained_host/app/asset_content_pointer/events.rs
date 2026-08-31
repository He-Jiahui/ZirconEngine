use super::super::RetainedEditorHost;

mod click;
mod motion;

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
        origin_x: f32,
        origin_y: f32,
    ) {
        if button == 1 && kind == 2 {
            self.active_asset_drag_payload = None;
            return;
        }
        if kind != 0 || button != 1 {
            if kind == 0 && button == 2 {
                self.dispatch_asset_content_context_menu(
                    surface_mode,
                    x,
                    y,
                    width,
                    height,
                    origin_x + x,
                    origin_y + y,
                );
            }
            return;
        }
        self.dispatch_asset_content_pointer_press(surface_mode, x, y, width, height);
    }
}
