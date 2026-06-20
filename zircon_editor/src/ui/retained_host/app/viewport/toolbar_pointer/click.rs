use super::super::super::{callback_dispatch, RetainedEditorHost};
use crate::ui::retained_host::viewport_toolbar_pointer::build_viewport_toolbar_pointer_layout_with_size;
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn viewport_toolbar_pointer_clicked(
        &mut self,
        surface_key: &str,
        control_id: &str,
        control_x: f32,
        control_y: f32,
        control_width: f32,
        control_height: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let surface_size = self.viewport_toolbar_surface_size(surface_key);
        let _ = self.viewport_toolbar_bridge.recompute_layout(surface_size);
        self.viewport_toolbar_pointer_bridge
            .sync(build_viewport_toolbar_pointer_layout_with_size(
                [surface_key],
                surface_size,
            ));
        match callback_dispatch::dispatch_shared_viewport_toolbar_pointer_click(
            &self.runtime,
            &self.viewport_toolbar_bridge,
            &mut self.viewport_toolbar_pointer_bridge,
            surface_key,
            control_id,
            control_x,
            control_y,
            control_width,
            control_height,
            UiPoint::new(point_x, point_y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
